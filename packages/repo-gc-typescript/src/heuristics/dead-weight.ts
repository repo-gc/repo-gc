import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc-shared';
import { SourceFile, FileInfo, ImportGraph } from '../types';

const CONFIG_PATTERNS = [
  /\.config\.(ts|js|mjs|cjs)$/,
  /\.eslintrc\.(js|cjs|json)$/,
  /\.prettierrc\.(js|cjs|json)$/,
  /tailwind\.config\.(ts|js)$/,
  /vite\.config\.(ts|js)$/,
  /jest\.config\.(ts|js)$/,
];

function isConfigFile(relativePath: string): boolean {
  return CONFIG_PATTERNS.some((p) => p.test(relativePath));
}

export function analyze(
  files: SourceFile[],
  infos: FileInfo[],
  graph: ImportGraph,
  idCounter: { value: number },
): Finding[] {
  const findings: Finding[] = [];

  // Build set of known entry points (from package.json main/bin/exports)
  const entryPoints = new Set<string>();
  for (const info of infos) {
    if (info.isEntryPoint) entryPoints.add(info.path);
  }

  for (const info of infos) {
    // Config files are never orphaned — they're consumed by tooling
    if (isConfigFile(info.relativePath)) continue;

    // Entry points are never orphaned
    if (entryPoints.has(info.path)) continue;

    const fanIn = graph.fanIn.get(info.path) || 0;
    if (fanIn > 0) continue;

    // Only flag if the file has meaningful content
    const file = files.find((f) => f.path === info.path);
    if (!file || file.lineCount < 10) continue;

    const lineCount = file.lineCount;
    let severity: Severity;
    if (lineCount >= 1000) severity = Severity.Critical;
    else if (lineCount >= 500) severity = Severity.High;
    else severity = Severity.Medium;

    findings.push({
      id: `dw-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.DeadWeight,
      severity,
      confidence: 0.6,
      path: info.relativePath,
      summary: `${lineCount}-line file is never imported`,
      reasons: [
        'No other file in the project imports this module',
        'It is not an entry point (package.json main/bin/exports) or config file',
      ],
      evidence: [`${lineCount} lines, fan-in=0`],
      suggested_next_step:
        'Remove the file if unused, or ensure it is reachable from an entry point',
    });
  }

  return findings;
}
