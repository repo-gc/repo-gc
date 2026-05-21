import { Finding, FindingKind, Severity, type FileData, type GraphData } from '../types';
import { estimateTokens } from '../token-estimate';

export interface DeadWeightOptions {
  /** File stem names that are always considered entry points (not orphaned). */
  entryPointStems?: string[];
  /** Regex patterns for config files that are never orphaned. */
  configPatterns?: RegExp[];
}

function resolveStem(relativePath: string): string {
  const parts = relativePath.split('/');
  const filename = parts[parts.length - 1];
  return filename.replace(/\.[^.]+$/, '');
}

export function analyzeDeadWeight(
  files: FileData[],
  graph: GraphData,
  idCounter: { value: number },
  options: DeadWeightOptions = {},
): Finding[] {
  const entryPointStems = options.entryPointStems ?? [];
  const configPatterns = options.configPatterns ?? [];

  // Build referenced-set from module declarations (Rust mod tree) + graph fan-in
  const referenced = new Set<string>();
  for (const f of files) {
    for (const decl of f.moduleDeclarations) {
      referenced.add(decl);
      // Add all prefixes so a reference to a.b.c also marks a and a.b
      const parts = decl.split('::');
      for (let i = 1; i < parts.length; i++) {
        referenced.add(parts.slice(0, i).join('::'));
      }
    }
  }

  const findings: Finding[] = [];

  for (const data of files) {
    // Entry points are never orphaned
    if (data.isEntryPoint) continue;

    // Package init files are never orphaned
    if (data.isPackageInit) continue;

    // Config files are never orphaned
    const stem = resolveStem(data.relativePath);
    if (entryPointStems.includes(stem)) continue;
    if (configPatterns.some((p) => p.test(data.relativePath))) continue;

    // Check if referenced via module declarations or import graph
    const fanIn = graph.fanIn.get(data.path) || 0;
    if (fanIn > 0) continue;
    if (referenced.has(data.modulePath)) continue;

    // Only flag if the file has meaningful content
    if (data.lineCount < 10) continue;

    let severity: Severity;
    if (data.lineCount >= 1000) severity = Severity.Critical;
    else if (data.lineCount >= 500) severity = Severity.High;
    else severity = Severity.Medium;

    const tokens = estimateTokens(data.sizeBytes);

    findings.push({
      id: `dw-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.DeadWeight,
      severity,
      confidence: 0.6,
      path: data.relativePath,
      summary: '',
      reasons: [],
      evidence: [
        `module_path: ${data.modulePath}`,
        `line_count: ${data.lineCount}`,
        `stem: ${stem}`,
        `estimated_tokens: ${tokens}`,
      ],
      suggested_next_step: '',
      estimated_tokens: tokens,
    });
  }

  return findings;
}
