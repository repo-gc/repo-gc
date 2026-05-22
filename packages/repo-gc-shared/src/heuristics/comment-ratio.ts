import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

export function analyzeCommentRatio(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const ratio = data.commentLineCount / Math.max(data.lineCount, 1);

  if (ratio < thresholds.commentRatioMin) {
    return {
      id: `cr-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.CommentRatio,
      severity: Severity.Medium,
      confidence: 0.65,
      path: data.relativePath,
      summary: '',
      reasons: [],
      evidence: [
        `comment_lines: ${data.commentLineCount}`,
        `total_lines: ${data.lineCount}`,
        `ratio: ${ratio.toFixed(4)}`,
        `direction: sparse`,
      ],
      suggested_next_step: '',
    };
  }

  if (ratio > thresholds.commentRatioMax) {
    return {
      id: `cr-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.CommentRatio,
      severity: Severity.Low,
      confidence: 0.65,
      path: data.relativePath,
      summary: '',
      reasons: [],
      evidence: [
        `comment_lines: ${data.commentLineCount}`,
        `total_lines: ${data.lineCount}`,
        `ratio: ${ratio.toFixed(4)}`,
        `direction: verbose`,
      ],
      suggested_next_step: '',
    };
  }

  return null;
}
