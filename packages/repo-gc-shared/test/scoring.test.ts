import { describe, it, expect } from 'vitest';
import { computeGlobalScore } from '../src/scoring';
import { Finding, FindingKind, Severity } from '../src/types';

function mkFinding(kind: FindingKind, severity: Severity, confidence = 1.0): Finding {
  return {
    id: 'x',
    kind,
    severity,
    confidence,
    path: 'src/test.ts',
    summary: '',
    reasons: [],
    evidence: [],
    suggested_next_step: '',
  };
}

describe('computeGlobalScore', () => {
  it('returns zero scores for no findings', () => {
    const score = computeGlobalScore([], 10, 0);
    expect(score.ai_friction_score).toBe(0);
    expect(score.context_waste_score).toBe(0);
    expect(score.structural_entropy_score).toBe(0);
  });

  it('caps scores at 100', () => {
    const findings = Array.from({ length: 100 }, () =>
      mkFinding(FindingKind.ContextBomb, Severity.Critical),
    );
    const score = computeGlobalScore(findings, 5, 0);
    expect(score.ai_friction_score).toBeLessThanOrEqual(100);
  });

  it('higher severity gives higher score', () => {
    const low = [mkFinding(FindingKind.ContextBomb, Severity.Low)];
    const high = [mkFinding(FindingKind.ContextBomb, Severity.Critical)];
    const lowScore = computeGlobalScore(low, 10, 0);
    const highScore = computeGlobalScore(high, 10, 0);
    expect(highScore.ai_friction_score).toBeGreaterThan(lowScore.ai_friction_score);
  });

  it('separates context waste from structural entropy', () => {
    const cb1 = mkFinding(FindingKind.ContextBomb, Severity.High);
    const cb2 = mkFinding(FindingKind.ContextBomb, Severity.High);
    const dw = mkFinding(FindingKind.DeadWeight, Severity.High);
    const score = computeGlobalScore([cb1, cb2, dw], 10, 0);
    expect(score.context_waste_score).toBeGreaterThan(0);
    expect(score.structural_entropy_score).toBeGreaterThan(0);
    expect(score.context_waste_score).toBeGreaterThan(score.structural_entropy_score);
  });

  it('scores all non-ContextBomb findings as structural entropy', () => {
    const kinds = [
      FindingKind.DeadWeight,
      FindingKind.ReexportEntropy,
      FindingKind.CouplingHotspot,
      FindingKind.CodeDuplication,
      FindingKind.UnusedImport,
    ];
    const findings = kinds.map((k) => mkFinding(k, Severity.Medium));
    const score = computeGlobalScore(findings, 10, 0);
    expect(score.structural_entropy_score).toBeGreaterThan(0);
    expect(score.context_waste_score).toBe(0);
  });

  it('computes context waste ratio', () => {
    const score = computeGlobalScore([], 1, 64000);
    expect(score.context_waste_ratio).toBe(0.5);
  });
});
