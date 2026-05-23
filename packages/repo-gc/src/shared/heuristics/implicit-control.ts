import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export interface ImplicitControlOptions {
  /**
   * Optional predicate that returns true for framework-heavy files
   * (e.g. NestJS, Angular, dataclass-heavy) that deserve a relaxed
   * decorator density limit. Kept as an injected callback so the
   * shared heuristic stays language-agnostic — each plugin defines
   * what "framework-heavy" means for its ecosystem.
   */
  isFrameworkHeavyFile?: (data: FileData) => boolean;
}

const FRAMEWORK_MULTIPLIER = 1.5;

export function analyzeImplicitControl(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
  options: ImplicitControlOptions = {},
): Finding | null {
  const decoratorCount = data.decoratorCount;
  const fnCount = Math.max(data.functionCount, 1);
  const ratio = decoratorCount / fnCount;

  const multiplier =
    options.isFrameworkHeavyFile?.(data) ? FRAMEWORK_MULTIPLIER : 1.0;
  const effectiveLimit = thresholds.decoratorDensityLimit * multiplier;

  if (ratio <= effectiveLimit) return null;

  let severity: Severity;
  if (ratio >= effectiveLimit * 3) {
    severity = Severity.High;
  } else if (ratio > effectiveLimit * 2) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  return {
    id: `ic-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ImplicitControl,
    severity,
    confidence: 0.40,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `decorator_count: ${decoratorCount}`,
      `function_count: ${fnCount}`,
      `ratio: ${ratio.toFixed(3)}`,
      `limit: ${effectiveLimit.toFixed(3)}`,
    ],
    suggested_next_step: '',
    estimated_tokens: undefined,
  };
}
