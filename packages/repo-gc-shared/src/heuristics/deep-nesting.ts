import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export function analyzeDeepNesting(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const limit = thresholds.nestingDepthLimit;
  if (data.maxNestingDepth <= limit) return null;

  const depth = data.maxNestingDepth;
  let severity: Severity;
  if (depth >= limit + 4) {
    severity = Severity.Critical;
  } else if (depth >= limit + 2) {
    severity = Severity.High;
  } else {
    severity = Severity.Medium;
  }

  return {
    id: `dn-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.DeepNesting,
    severity,
    confidence: 0.85,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `max_depth: ${depth}`,
      `limit: ${limit}`,
      `deepest_at: unknown`,
    ],
    suggested_next_step: '',
    estimated_tokens: undefined,
  };
}
