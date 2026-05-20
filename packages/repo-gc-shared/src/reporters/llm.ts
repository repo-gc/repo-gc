import { FINDING_KIND_LLM, SEVERITY_LLM, type Report, type Finding, FindingKind } from '../types';

function evVal(evidence: string[], key: string): string {
  const prefix = `${key}: `;
  for (const e of evidence) {
    if (e.startsWith(prefix)) return e.slice(prefix.length);
  }
  return '-';
}

function compactSummary(f: Finding): string {
  switch (f.kind) {
    case FindingKind.ContextBomb: {
      const lc = evVal(f.evidence, 'line_count');
      const tk = evVal(f.evidence, 'estimated_tokens');
      let s = `${lc}ln/${tk}tk`;
      for (const r of f.reasons) {
        if (r.includes('functions defined')) {
          const n = r.split(/\s+/)[0];
          s += ` +${n}fn`;
        } else if (r.includes('impl blocks')) {
          const n = r.split(/\s+/)[0];
          s += ` +${n}impl`;
        }
      }
      return s;
    }
    case FindingKind.CouplingHotspot: {
      const fi = evVal(f.evidence, 'fan_in');
      const fo = evVal(f.evidence, 'fan_out');
      const istab = evVal(f.evidence, 'instability');
      const pat = evVal(f.evidence, 'pattern');
      return `in=${fi} out=${fo} I=${istab} ${pat}`;
    }
    case FindingKind.DeadWeight: {
      const mp = evVal(f.evidence, 'module_path');
      const lc = evVal(f.evidence, 'line_count');
      return `${lc}ln mod=${mp}`;
    }
    case FindingKind.ReexportEntropy: {
      const pu = evVal(f.evidence, 'pub_use_count');
      const ti = evVal(f.evidence, 'total_reexported_items');
      const wc = evVal(f.evidence, 'has_wildcard');
      const w = wc === 'true' ? ' +*' : '';
      return `${ti}sym/${pu}pu${w}`;
    }
    case FindingKind.CodeDuplication: {
      const files = f.evidence.length;
      const fnName = f.evidence[0]?.split(' :: ')[1] || '?';
      return `fn:${fnName} x${files}`;
    }
    case FindingKind.UnusedImport: {
      const n = f.evidence.length;
      const names = f.evidence
        .filter(e => e.startsWith('imported but unreferenced: '))
        .map(e => e.slice('imported but unreferenced: '.length))
        .slice(0, 4);
      return `${n}: ${names.join(',')}`;
    }
    default:
      return '-';
  }
}

function compactNext(f: Finding): string {
  switch (f.kind) {
    case FindingKind.ContextBomb: {
      const lc = evVal(f.evidence, 'line_count');
      return `split <${lc}ln`;
    }
    case FindingKind.CouplingHotspot: {
      const pat = evVal(f.evidence, 'pattern');
      if (pat === 'api') return 'verify api';
      if (pat === 'orch') return 'split deps';
      return 'decouple';
    }
    case FindingKind.DeadWeight: return 'rm or re-export';
    case FindingKind.ReexportEntropy: return 'flatten re-exports';
    case FindingKind.CodeDuplication: return 'DRY: shared util';
    case FindingKind.UnusedImport: return 'rm imports';
    default: return '-';
  }
}

export function renderLlm(report: Report): string {
  const lines: string[] = [];
  const gs = report.global_score;

  const header = [
    `ver=${report.version}`,
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
  if (report.errors.length > 0) {
    lines.push(`err=${report.errors.join('; ')}`);
  }

  if (report.findings.length > 0) {
    lines.push('id\tsev\tkind\tpath\ttok\tsummary\tnext');
  }

  for (const f of report.findings) {
    const sev = SEVERITY_LLM[f.severity];
    const kind = FINDING_KIND_LLM[f.kind];
    const tok = f.estimated_tokens ? `${f.estimated_tokens}tk` : '-';
    const summary = compactSummary(f);
    const next = compactNext(f);
    lines.push([f.id, sev, kind, f.path, tok, summary, next].join('\t'));
  }

  return lines.join('\n');
}
