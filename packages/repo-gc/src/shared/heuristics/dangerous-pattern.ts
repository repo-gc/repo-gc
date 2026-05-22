import { Finding, FindingKind, Severity, type FileData } from '../types';
import { type Thresholds } from '../threshold';

export function analyzeDangerousPattern(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const count = data.dangerousPatternCount;
  const limit = thresholds.dangerousPatternLimit;
  if (count <= limit) return null;

  let severity: Severity;
  if (count >= limit * 3) {
    severity = Severity.Critical;
  } else if (count >= limit * 2) {
    severity = Severity.High;
  } else {
    severity = Severity.Medium;
  }

  return {
    id: `dp-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.DangerousPattern,
    severity,
    confidence: 0.70,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `dangerous_pattern_count: ${count}`,
      `limit: ${limit}`,
      `preview: ${data.relativePath}`,
    ],
    suggested_next_step: '',
    estimated_tokens: undefined,
  };
}
