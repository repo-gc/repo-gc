import { Finding, FindingKind, GlobalScore, SEVERITY_WEIGHT } from './types';

export function computeGlobalScore(
  findings: Finding[],
  totalFiles: number,
  totalEstimatedTokens: number,
): GlobalScore {
  const maxExpected = Math.max(totalFiles * 0.4, 1.0);

  const sumWeight = (ff: Finding[]) =>
    ff.reduce((acc, f) => acc + SEVERITY_WEIGHT[f.severity] * f.confidence, 0);

  const clamp = (v: number) => Math.min(Math.round(v), 100);

  const aiFriction = clamp((sumWeight(findings) / maxExpected) * 100);

  const contextWaste = clamp(
    (sumWeight(findings.filter((f) => f.kind === FindingKind.ContextBomb)) / maxExpected) * 100,
  );

  const structuralEntropy = clamp(
    (sumWeight(
      findings.filter(
        (f) =>
          f.kind === FindingKind.ReexportEntropy ||
          f.kind === FindingKind.CouplingHotspot ||
          f.kind === FindingKind.DeadWeight ||
          f.kind === FindingKind.CodeDuplication ||
          f.kind === FindingKind.UnusedImport,
      ),
    ) /
      maxExpected) *
      100,
  );

  const contextWindowTokens = 128_000;
  const contextWasteRatio = totalEstimatedTokens / contextWindowTokens;
  const estimatedWastePct = clamp(
    (totalEstimatedTokens / Math.max(totalFiles * contextWindowTokens, 1)) * 100,
  );

  return {
    ai_friction_score: aiFriction,
    context_waste_score: contextWaste,
    structural_entropy_score: structuralEntropy,
    context_waste_ratio: contextWasteRatio,
    estimated_waste_pct: estimatedWastePct,
  };
}
