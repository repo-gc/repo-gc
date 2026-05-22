import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export function analyzeTypeComplexity(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const depth = data.maxTypeDepth;
  const limit = thresholds.typeDepthLimit;

  if (depth <= limit) return null;

  let severity: Severity;
  if (depth >= limit + 3) {
    severity = Severity.Critical;
  } else if (depth >= limit + 2) {
    severity = Severity.High;
  } else {
    severity = Severity.Medium;
  }

  return {
    id: `tc-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.TypeComplexity,
    severity,
    confidence: 0.50,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `max_type_depth: ${depth}`,
      `limit: ${limit}`,
      `location: unknown`,
    ],
    suggested_next_step: '',
    estimated_tokens: undefined,
  };
}
