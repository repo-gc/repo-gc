import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export function analyzeImplicitControl(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const decoratorCount = data.decoratorCount;
  const fnCount = Math.max(data.functionCount, 1);
  const ratio = decoratorCount / fnCount;

  // Framework-aware: NestJS/Angular/dataclass-heavy files get 1.5x threshold
  const effectiveLimit = isFrameworkHeavyFile(data)
    ? thresholds.decoratorDensityLimit * 1.5
    : thresholds.decoratorDensityLimit;

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

function isFrameworkHeavyFile(data: FileData): boolean {
  const frameworkPatterns = [
    /@nestjs\//, /@angular\//, /@Component/, /@NgModule/,
    /@Entity/, /@Injectable/, /@Controller/, /@Service/,
    /@dataclass/, /@pydantic/,
  ];
  for (const imp of data.imports) {
    for (const pat of frameworkPatterns) {
      if (pat.test(imp)) return true;
    }
  }
  return false;
}
