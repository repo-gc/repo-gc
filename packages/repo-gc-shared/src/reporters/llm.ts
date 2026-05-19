import { FINDING_KIND_LLM, SEVERITY_LLM, type Report, type Finding } from '../types';

export function renderLlm(report: Report): string {
  const lines: string[] = [];
  const gs = report.global_score;

  const header = [
    `fric=${gs.ai_friction_score}`,
    `waste=${gs.context_waste_score}`,
    `ent=${gs.structural_entropy_score}`,
    `ratio=${gs.context_waste_ratio.toFixed(1)}`,
    `pct=${gs.estimated_waste_pct}`,
    `files=${report.files_analyzed}`,
    `skip=${report.files_skipped}`,
    `tok=${report.total_estimated_tokens}`,
  ].join(' ');
  lines.push(header);

  for (const f of report.findings) {
    const sev = SEVERITY_LLM[f.severity];
    const kind = FINDING_KIND_LLM[f.kind];
    const tok = f.estimated_tokens ? `${f.estimated_tokens}tk` : '-';
    const reasons = f.reasons.join('; ');
    const next = f.suggested_next_step;
    lines.push([f.id, sev, kind, f.path, tok, f.summary, reasons, next].join('\t'));
  }

  return lines.join('\n');
}
