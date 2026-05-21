import { Finding, FindingKind, Severity, type FileData } from '../types';
import { type Thresholds } from '../threshold';
import { estimateTokens } from '../token-estimate';

export function analyzeContextBombs(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  if (data.lineCount < thresholds.lineCountLimit) return null;

  const ratio = data.lineCount / thresholds.lineCountLimit;
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

  const tokens = estimateTokens(data.sizeBytes);
  const evidence = [
    `line_count: ${data.lineCount}`,
    `estimated_tokens: ${tokens}`,
    `limit: ${thresholds.lineCountLimit}`,
  ];
  if (data.functionCount > 10) evidence.push(`function_count: ${data.functionCount}`);
  if (data.classCount > 3) evidence.push(`class_count: ${data.classCount}`);
  if (data.implBlockCount > 3) evidence.push(`impl_block_count: ${data.implBlockCount}`);

  return {
    id: `cb-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ContextBomb,
    severity,
    confidence,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence,
    suggested_next_step: '',
    estimated_tokens: tokens,
  };
}
