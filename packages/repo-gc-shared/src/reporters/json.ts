import { type Report } from '../types';

export function renderJson(report: Report): string {
  const output = {
    version: report.version,
    findings: report.findings.map((f) => ({
      id: f.id,
      kind: f.kind,
      severity: f.severity,
      confidence: f.confidence,
      path: f.path,
      summary: f.summary,
      reasons: f.reasons,
      evidence: f.evidence,
      suggested_next_step: f.suggested_next_step,
      estimated_tokens: f.estimated_tokens ?? null,
    })),
    global_score: {
      ai_friction_score: report.global_score.ai_friction_score,
      context_waste_score: report.global_score.context_waste_score,
      structural_entropy_score: report.global_score.structural_entropy_score,
      context_waste_ratio: report.global_score.context_waste_ratio,
      estimated_waste_pct: report.global_score.estimated_waste_pct,
    },
    files_analyzed: report.files_analyzed,
    files_skipped: report.files_skipped,
    total_lines: report.total_lines,
    total_estimated_tokens: report.total_estimated_tokens,
    errors: report.errors,
  };
  return JSON.stringify(output, null, 2);
}
