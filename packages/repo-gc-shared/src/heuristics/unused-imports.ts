import { Finding, FindingKind, Severity, type FileData } from '../types';

export interface UnusedImportsOptions {
  /** Names commonly referenced by compile-time transforms rather than user code. */
  compilerNames?: Set<string>;
}

export function analyzeUnusedImports(
  data: FileData,
  idCounter: { value: number },
  options: UnusedImportsOptions = {},
): Finding | null {
  const compilerNames = options.compilerNames ?? new Set<string>();

  const unusedNames: string[] = [];
  for (const name of data.importedNames) {
    if (compilerNames.has(name)) continue;
    if (data.allIdentifiers.has(name)) continue;
    unusedNames.push(name);
  }

  // Require at least 2 unused names to reduce false positives from macros/transforms
  if (unusedNames.length < 2) return null;

  const severity = unusedNames.length >= 5 ? Severity.Medium : Severity.Low;

  return {
    id: `ui-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.UnusedImport,
    severity,
    confidence: 0.65,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `unused_count: ${unusedNames.length}`,
      `preview: ${unusedNames.slice(0, 5).join(', ')}`,
    ],
    suggested_next_step: '',
  };
}
