import { FindingKind, type Report, type Severity } from '../types';

const KIND_HEADER: Record<FindingKind, string> = {
  [FindingKind.ContextBomb]: 'Context Bombs (Oversized Files)',
  [FindingKind.DeadWeight]: 'Dead Weight (Unreferenced Modules)',
  [FindingKind.ReexportEntropy]: 'Re-export Entropy (Barrel Files)',
  [FindingKind.CouplingHotspot]: 'Coupling Hotspots',
  [FindingKind.CodeDuplication]: 'Code Duplication',
  [FindingKind.UnusedImport]: 'Unused Imports',
};

export function renderMarkdown(report: Report): string {
  const lines: string[] = [];
  const gs = report.global_score;

  lines.push('# AI Context Efficiency Report');
  lines.push('');
  lines.push(`> repo-gc v${report.version}`);
  lines.push('');
  lines.push('## Summary');
  lines.push('');
  lines.push('| Metric | Value |');
  lines.push('|--------|-------|');
  lines.push(`| Files analyzed | ${report.files_analyzed} |`);
  lines.push(`| Files skipped | ${report.files_skipped} |`);
  lines.push(`| Total lines | ${report.total_lines} |`);
  lines.push(`| Total estimated tokens | ${report.total_estimated_tokens} |`);
  lines.push('');
  lines.push('## Scores');
  lines.push('');
  lines.push('| Score | Value |');
  lines.push('|-------|-------|');
  lines.push(`| AI Friction | ${gs.ai_friction_score}/100 |`);
  lines.push(`| Context Waste | ${gs.context_waste_score}/100 |`);
  lines.push(`| Structural Entropy | ${gs.structural_entropy_score}/100 |`);
  lines.push(`| Context Waste Ratio | ${gs.context_waste_ratio.toFixed(1)}x |`);
  lines.push(`| Estimated Waste | ${gs.estimated_waste_pct}% |`);
  lines.push('');

  if (report.findings.length === 0) {
    lines.push('## No issues found');
    lines.push('');
    lines.push('Your repository is AI-ready. No context-wasting patterns detected.');
  } else {
    lines.push('## Findings');
    lines.push('');

    const grouped = new Map<FindingKind, typeof report.findings>();
    for (const f of report.findings) {
      const list = grouped.get(f.kind) || [];
      list.push(f);
      grouped.set(f.kind, list);
    }

    for (const [kind, findings] of grouped) {
      lines.push(`### ${KIND_HEADER[kind]} (${findings.length})`);
      lines.push('');
      for (const f of findings) {
        lines.push(`- **\`${f.path}\`** [${f.severity}] — ${f.summary}`);
        lines.push(`  - ${f.suggested_next_step}`);
      }
      lines.push('');
    }
  }

  if (report.errors.length > 0) {
    lines.push('## Errors');
    lines.push('');
    for (const err of report.errors) {
      lines.push(`- ${err}`);
    }
    lines.push('');
  }

  return lines.join('\n');
}
