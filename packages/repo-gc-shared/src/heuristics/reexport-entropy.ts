import { Finding, FindingKind, Severity, type Thresholds, type FileData } from '../types';

export function analyzeReexportEntropy(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  // Count non-local re-exports
  const reExports = data.exports.filter((e) => e.sourcePath !== '<local>');

  // For package init files, all imports count as re-exports (Python barrel pattern)
  // For non-init files with __all__, that also counts as a re-export surface
  const hasAllExport = data.allExport !== null && data.allExport.length > 0;
  const wildcardCount = reExports.filter((e) => e.isWildcard).length;

  let effectiveReexportCount = reExports.length;
  let totalItems = reExports.reduce((sum, e) => sum + (e.isWildcard ? 0 : e.itemCount), 0);

  if (data.isPackageInit && effectiveReexportCount < 3) {
    // Init files: imported names themselves are the re-export surface
    effectiveReexportCount = data.importedNames.length;
    totalItems = data.importedNames.length;
  }

  if (hasAllExport && !data.isPackageInit) {
    // Non-init file with __all__: treat as a re-export barrel
    effectiveReexportCount = Math.max(effectiveReexportCount, 1);
    totalItems = Math.max(totalItems, data.allExport!.length);
  }

  if (effectiveReexportCount < 3) return null;
  if (totalItems < thresholds.reexportLimit && wildcardCount === 0) return null;

  let severity: Severity;
  const ratio = totalItems / thresholds.reexportLimit;

  if (wildcardCount > 0 || ratio >= 3) {
    severity = Severity.High;
  } else if (totalItems >= thresholds.reexportLimit) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  return {
    id: `re-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ReexportEntropy,
    severity,
    confidence: 0.8,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `reexport_count: ${effectiveReexportCount}`,
      `total_items: ${totalItems}`,
      `has_wildcard: ${wildcardCount > 0}`,
    ],
    suggested_next_step: '',
  };
}
