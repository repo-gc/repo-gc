import { Finding, FindingKind, Severity, type FileData } from '../types';
import { type Thresholds } from '../threshold';

export function analyzeErrorSwallow(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const count = data.emptyCatchCount;
  if (count <= thresholds.emptyCatchLimit) return null;

  let severity: Severity;
  if (count >= 5) {
    severity = Severity.High;
  } else if (count >= 3) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  return {
    id: `es-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ErrorSwallow,
    severity,
    confidence: 0.75,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `empty_catch_count: ${count}`,
      `limit: ${thresholds.emptyCatchLimit}`,
      `preview: ${data.relativePath}`,
    ],
    suggested_next_step: '',
  };
}
