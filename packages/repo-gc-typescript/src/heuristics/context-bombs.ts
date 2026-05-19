import { Finding, FindingKind, Severity, estimateTokens, type Thresholds } from 'repo-gc-shared';
import { SourceFile, FileInfo } from '../types';

export function analyze(
  file: SourceFile,
  _info: FileInfo,
  threshold: Thresholds,
  idCounter: { value: number },
): Finding | null {
  if (file.lineCount < threshold.lineCountLimit) return null;

  const ratio = file.lineCount / threshold.lineCountLimit;
  let severity: Severity;
  let confidence: number;
  if (ratio >= 4) {
    severity = Severity.Critical;
    confidence = 0.95;
  } else if (ratio >= 2) {
    severity = Severity.High;
    confidence = 0.85;
  } else {
    severity = Severity.Medium;
    confidence = 0.75;
  }

  const tokens = estimateTokens(file.sizeBytes);
  const reasons: string[] = [];
  if (ratio >= 4) reasons.push(`File is ${ratio.toFixed(0)}x over the ${threshold.lineCountLimit}-line limit`);
  if (tokens > 4000) reasons.push(`Estimated ${tokens} tokens — eats ${((tokens / 128000) * 100).toFixed(1)}% of a Claude session window`);

  const evidence = [`${file.lineCount}ln / ${tokens}tk`];
  if (file.sizeBytes > 50000) evidence.push(`${(file.sizeBytes / 1024).toFixed(1)}KB source file`);

  return {
    id: `cb-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ContextBomb,
    severity,
    confidence,
    path: file.relativePath,
    summary: `${file.lineCount}-line file (${tokens} estimated tokens)`,
    reasons,
    evidence,
    suggested_next_step: `Split into smaller modules targeting <${threshold.lineCountLimit} lines each`,
    estimated_tokens: tokens,
  };
}
