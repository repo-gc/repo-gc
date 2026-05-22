import picocolors from 'picocolors';
import { Severity, FindingKind, type Report, type Finding } from '../types';

const { bold, green, yellow, red, cyan, gray, dim, white, bgRed, bgYellow, bgGreen } = picocolors;

const SEV_COLOR: Record<Severity, (s: string) => string> = {
  [Severity.Critical]: (s) => bgRed(white(bold(s))),
  [Severity.High]: red,
  [Severity.Medium]: yellow,
  [Severity.Low]: dim,
};

const KIND_LABEL: Record<FindingKind, string> = {
  [FindingKind.ContextBomb]: 'Context Bomb',
  [FindingKind.DeadWeight]: 'Dead Weight',
  [FindingKind.ReexportEntropy]: 'Re-export Entropy',
  [FindingKind.CouplingHotspot]: 'Coupling Hotspot',
  [FindingKind.CodeDuplication]: 'Code Duplication',
  [FindingKind.UnusedImport]: 'Unused Import',
  [FindingKind.BranchDensity]: 'Branch Density',
  [FindingKind.DeepNesting]: 'Deep Nesting',
  [FindingKind.TypeComplexity]: 'Type Complexity',
  [FindingKind.CommentRatio]: 'Comment Ratio',
  [FindingKind.ImplicitControl]: 'Implicit Control',
  [FindingKind.ErrorSwallow]: 'Error Swallow',
  [FindingKind.DangerousPattern]: 'Dangerous Pattern',
  [FindingKind.MutableGlobal]: 'Mutable Global',
  [FindingKind.NamingEntropy]: 'Naming Entropy',
  [FindingKind.StringlyTyped]: 'Stringly Typed',
  [FindingKind.ImportDiversity]: 'Import Diversity',
  [FindingKind.PlatformDensity]: 'Platform Density',
};

function scoreBar(score: number): string {
  const filled = Math.round(score / 10);
  const empty = 10 - filled;
  let bar = '';
  for (let i = 0; i < filled; i++) bar += i < 4 ? green('█') : i < 7 ? yellow('█') : red('█');
  for (let i = 0; i < empty; i++) bar += gray('░');
  return bar;
}

export function renderTerminal(report: Report, color: boolean): string {
  const c = color ? picocolors : picocolors.createColors(false);
  const lines: string[] = [];

  const gs = report.global_score;

  lines.push('');
  lines.push(c.bold('═══ AI Context Efficiency Report ═══'));
  lines.push(c.dim(`  repo-gc v${report.version}`));
  lines.push('');
  lines.push(`  Files analyzed: ${report.files_analyzed}  |  Skipped: ${report.files_skipped}  |  Total lines: ${report.total_lines}`);
  lines.push(`  Total estimated tokens: ${report.total_estimated_tokens}  (${gs.context_waste_ratio.toFixed(1)}x Claude session)`);
  lines.push('');

  lines.push(c.bold('Scores'));
  lines.push(`  AI Friction:        ${scoreBar(gs.ai_friction_score)} ${gs.ai_friction_score}/100`);
  lines.push(`  Context Waste:      ${scoreBar(gs.context_waste_score)} ${gs.context_waste_score}/100`);
  lines.push(`  Structural Entropy: ${scoreBar(gs.structural_entropy_score)} ${gs.structural_entropy_score}/100`);
  lines.push(`  Est. Waste:         ${gs.estimated_waste_pct}% of context window`);
  lines.push('');

  const grouped = new Map<FindingKind, Finding[]>();
  for (const f of report.findings) {
    const list = grouped.get(f.kind) || [];
    list.push(f);
    grouped.set(f.kind, list);
  }

  if (report.findings.length === 0) {
    lines.push(c.green('  No issues found. Your repo is AI-ready.'));
  } else {
    lines.push(c.bold(`Findings (${report.findings.length})`));
    for (const [kind, findings] of grouped) {
      lines.push('');
      lines.push(`  ${c.bold(KIND_LABEL[kind])} (${findings.length})`);
      for (const f of findings.slice(0, 5)) {
        const sevLabel = SEV_COLOR[f.severity](`[${f.severity[0]}]`);
        lines.push(`    ${sevLabel} ${f.path} — ${f.summary}`);
      }
      if (findings.length > 5) {
        lines.push(`    ${c.dim(`...and ${findings.length - 5} more`)}`);
      }
    }
  }

  if (report.errors.length > 0) {
    lines.push('');
    lines.push(c.bold(c.red('Errors')));
    for (const err of report.errors) {
      lines.push(`  ${c.red('✗')} ${c.dim(err)}`);
    }
  }

  const priority = report.findings
    .filter((f) => f.severity === Severity.Critical || f.severity === Severity.High)
    .slice(0, 5);
  if (priority.length > 0) {
    lines.push('');
    lines.push(c.bold('Priority Cleanup Targets'));
    for (const f of priority) {
      lines.push(`  ${c.red('▶')} ${f.path}`);
      lines.push(`    ${c.dim(f.suggested_next_step)}`);
    }
  }

  lines.push('');
  return lines.join('\n');
}
