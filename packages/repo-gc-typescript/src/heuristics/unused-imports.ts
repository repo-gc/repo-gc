import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc-shared';
import { FileInfo } from '../types';

// Names commonly referenced by compile-time transforms rather than user code
const COMPILER_NAMES = new Set([
  'React',       // old JSX transform
  'Fragment',    // JSX
  'jsx',         // new JSX transform
  'jsxs',        // new JSX transform (static)
]);

export function analyze(
  info: FileInfo,
  threshold: Thresholds,
  idCounter: { value: number },
): Finding | null {
  // Map each imported name to whether it's used
  const unusedNames: string[] = [];

  for (const name of info.importedNames) {
    if (COMPILER_NAMES.has(name)) continue; // skip compiler-injected usage
    if (info.allIdentifiers.has(name)) continue; // found in body
    unusedNames.push(name);
  }

  // Require at least 2 unused names to reduce false positives from derives/macros/transforms
  if (unusedNames.length < 2) return null;

  let severity: Severity;
  if (unusedNames.length >= 5) severity = Severity.Medium;
  else severity = Severity.Low;

  return {
    id: `ui-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.UnusedImport,
    severity,
    confidence: 0.65,
    path: info.relativePath,
    summary: `${unusedNames.length} unused imports: ${unusedNames.slice(0, 5).join(', ')}`,
    reasons: [
      `These names are imported but never referenced in the file body`,
      'If needed by a macro, derive, or JSX transform, they can be ignored',
    ],
    evidence: unusedNames.map((n) => `"${n}" imported but unused`),
    suggested_next_step:
      'Remove the unused imports, or verify they are needed by compile-time transforms',
  };
}
