import { Finding, FindingKind, Severity } from 'repo-gc';
import { FileInfo } from '../types';

export function analyze(infos: FileInfo[], idCounter: { value: number }): Finding[] {
  const findings: Finding[] = [];
  const seen = new Map<string, { name: string; paths: string[] }>();

  // Collect all function bodies across all files
  for (const info of infos) {
    for (const [fnName, body] of info.functionBodies) {
      if (seen.has(body)) {
        const entry = seen.get(body)!;
        if (!entry.paths.includes(info.relativePath)) {
          entry.paths.push(info.relativePath);
        }
      } else {
        seen.set(body, { name: fnName, paths: [info.relativePath] });
      }
    }
  }

  // Flag duplicates across 2+ distinct files
  for (const [body, entry] of seen) {
    const distinctFiles = [...new Set(entry.paths)];
    if (distinctFiles.length < 2) continue;

    let severity: Severity;
    if (distinctFiles.length >= 5) severity = Severity.High;
    else if (distinctFiles.length >= 3) severity = Severity.Medium;
    else severity = Severity.Low;

    findings.push({
      id: `dup-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.CodeDuplication,
      severity,
      confidence: 0.85,
      path: distinctFiles[0],
      summary: '',
      reasons: [],
      evidence: [
        `file_count: ${distinctFiles.length}`,
        `fn_name: ${entry.name}`,
        `files: ${distinctFiles.slice(0, 5).join(', ')}`,
      ],
      suggested_next_step: '',
    });
  }

  return findings;
}
