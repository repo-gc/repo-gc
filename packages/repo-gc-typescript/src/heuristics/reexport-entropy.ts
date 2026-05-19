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

  const reasons: string[] = [
    `${reExports.length} re-export declarations (${totalItems} total items)`,
  ];
  if (wildcardCount > 0) {
    reasons.push(`${wildcardCount} wildcard re-exports (export * from) hide the actual public API`);
  }

  const evidence = [`${reExports.length} re-exports, ${totalItems} items`];
  if (wildcardCount > 0) evidence.push(`${wildcardCount} wildcards`);

  return {
    id: `rx-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ReexportEntropy,
    severity,
    confidence: 0.8,
    path: info.relativePath,
    summary: `Barrel file with ${reExports.length} re-exports (${totalItems} items)`,
    reasons,
    evidence,
    suggested_next_step: 'Flatten the re-export chain — barrel files degrade LLM path resolution',
  };
}
