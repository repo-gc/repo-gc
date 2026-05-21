import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc-shared';
import { FileInfo } from '../types';

export function analyze(
  info: FileInfo,
  threshold: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const reExports = info.exports.filter((e) => e.sourcePath !== '<local>');

  if (reExports.length < 3) return null;

  const totalItems = reExports.reduce((sum, e) => sum + (e.isWildcard ? 0 : e.itemCount), 0);
  const wildcardCount = reExports.filter((e) => e.isWildcard).length;

  if (totalItems < threshold.reexportLimit && wildcardCount === 0) return null;

  let severity: Severity;
  const ratio = totalItems / threshold.reexportLimit;

  if (wildcardCount > 0 || ratio >= 3) {
    severity = Severity.High;
  } else if (totalItems >= threshold.reexportLimit) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  const evidence = [
    `reexport_count: ${reExports.length}`,
    `total_items: ${totalItems}`,
    `has_wildcard: ${wildcardCount > 0}`,
  ];

  return {
    id: `rx-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ReexportEntropy,
    severity,
    confidence: 0.8,
    path: info.relativePath,
    summary: '',
    reasons: [],
    evidence,
    suggested_next_step: '',
  };
}
