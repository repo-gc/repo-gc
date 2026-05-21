import { Finding, FindingKind, Severity, type FileData } from '../types';

export interface DuplicationOptions {
  /** Patterns for function names that are test-only and should be skipped. */
  testNamePatterns?: string[];
}

/** Minimum normalized body length to consider for duplication detection. */
const MIN_BODY_LEN = 40;

function isTestOnlyName(name: string, patterns: string[]): boolean {
  return patterns.some((p) => name.includes(p));
}

/**
 * Type 2 clone detection.
 *
 * Plugins pre-normalize function bodies (identifier replacement + whitespace strip).
 * This function groups by normalized body key, counts distinct files per group,
 * and emits findings for groups with >= 2 distinct files.
 */
export function analyzeDuplication(
  files: FileData[],
  idCounter: { value: number },
  options: DuplicationOptions = {},
): Finding[] {
  const testNamePatterns = options.testNamePatterns ?? ['for_test'];

  const seen = new Map<string, { name: string; paths: string[] }>();

  for (const file of files) {
    for (const fn of file.functionBodies) {
      if (isTestOnlyName(fn.name, testNamePatterns)) continue;

      const key = fn.normalizedBody;
      if (key.length < MIN_BODY_LEN) continue;

      const existing = seen.get(key);
      if (existing) {
        if (!existing.paths.includes(file.relativePath)) {
          existing.paths.push(file.relativePath);
        }
      } else {
        seen.set(key, { name: fn.name, paths: [file.relativePath] });
      }
    }
  }

  const findings: Finding[] = [];

  for (const [, entry] of seen) {
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
