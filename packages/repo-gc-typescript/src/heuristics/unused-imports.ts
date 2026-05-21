import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc';
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
    summary: '',
    reasons: [],
    evidence: [
      `unused_count: ${unusedNames.length}`,
      `preview: ${unusedNames.slice(0, 5).join(', ')}`,
    ],
    suggested_next_step: '',
  };
}
