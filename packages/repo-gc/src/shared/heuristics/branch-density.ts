import { Finding, FindingKind, Severity, type FileData } from '../types';
import { type Thresholds } from '../threshold';

export function analyzeBranchDensity(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const fnCount = Math.max(data.functionCount, 1);
  const ratio = data.branchCount / fnCount;
  const limit = thresholds.branchDensityLimit;

  if (ratio <= limit) return null;

  let severity: Severity;
  if (ratio >= limit * 3) {
    severity = Severity.Critical;
  } else if (ratio >= limit * 2) {
    severity = Severity.High;
  } else {
    severity = Severity.Medium;
  }

  return {
    id: `bd-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.BranchDensity,
    severity,
    confidence: 0.85,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `branch_count: ${data.branchCount}`,
      `function_count: ${fnCount}`,
      `avg_branches_per_fn: ${ratio.toFixed(2)}`,
      `limit: ${limit}`,
    ],
    suggested_next_step: '',
  };
}
