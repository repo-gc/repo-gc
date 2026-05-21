/** Estimate token count from byte size. Standard 4-bytes-per-token heuristic. */
export function estimateTokens(sizeBytes: number): number {
  return Math.round(sizeBytes / 4);
}
