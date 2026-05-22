import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export function analyzeStringlyTyped(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const count = data.stringComparisonCount;
  const limit = thresholds.stringComparisonLimit;
  if (count <= limit) return null;

  let severity: Severity;
  if (count >= limit * 3) {
    severity = Severity.High;
  } else if (count >= limit * 2) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  return {
    id: `st-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.StringlyTyped,
    severity,
    confidence: 0.65,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `string_comparison_count: ${count}`,
      `limit: ${limit}`,
      `preview: ${data.relativePath}`,
    ],
    suggested_next_step: '',
  };
}
