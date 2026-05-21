import { Finding, FindingKind, Severity, estimateTokens, type Thresholds } from 'repo-gc';
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

  const evidence = [
    `line_count: ${file.lineCount}`,
    `estimated_tokens: ${tokens}`,
    `limit: ${threshold.lineCountLimit}`,
  ];

  return {
    id: `cb-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ContextBomb,
    severity,
    confidence,
    path: file.relativePath,
    summary: '',
    reasons: [],
    evidence,
    suggested_next_step: '',
    estimated_tokens: tokens,
  };
}
