import { Finding, FindingKind } from './shared/types';

function parseEvidence(evidence: string[]): Record<string, string> {
  const data: Record<string, string> = {};
  for (const e of evidence) {
    const idx = e.indexOf(': ');
    if (idx === -1) continue;
    data[e.slice(0, idx)] = e.slice(idx + 2);
  }
  return data;
}

function enrichFinding(f: Finding): void {
  const d = parseEvidence(f.evidence);

  switch (f.kind) {
    case FindingKind.ContextBomb: {
      const lines = d.line_count || '?';
      const tokens = d.estimated_tokens || '?';
      const limit = d.limit || '?';

      f.summary = `Oversized file — ${lines} lines / ~${tokens} tokens, each AI edit re-reads this entire file`;

      f.reasons = [`${lines} lines (limit: ${limit})`];
      if (d.function_count) f.reasons.push(`${d.function_count} functions defined`);
      if (d.impl_block_count) f.reasons.push(`${d.impl_block_count} impl blocks`);
      if (d.class_count) f.reasons.push(`${d.class_count} classes defined`);

      f.suggested_next_step = `Split into smaller modules (target <${limit} lines each)`;
      break;
    }

    case FindingKind.DeadWeight: {
      const lines = d.line_count || '?';
      const modulePath = d.module_path || f.path;

      f.summary = `Dead module — ~${lines} lines loaded into agent context but never referenced`;

      f.reasons = [
        `Module '${modulePath}' is not imported by any other module — wasted context capacity`,
      ];

      f.suggested_next_step =
        'Remove the file if unused, or ensure it is reachable from an entry point';
      break;
    }

    case FindingKind.UnusedImport: {
      const unusedCount = d.unused_count || '?';
      const preview = d.preview || '';

      f.summary = `Zombie imports — ${unusedCount} unused names inflate token usage in every context window: ${preview}`;

      f.reasons = [
        `${unusedCount} imported names not referenced in file body`,
        'May be needed by compiler transforms, macros, or dynamic patterns — verify before removing',
      ];

      f.suggested_next_step =
        'Remove zombie imports to reduce token waste, or verify they are needed by language-specific transforms';
      break;
    }

    case FindingKind.CouplingHotspot: {
      const fanIn = d.fan_in || '0';
      const fanOut = d.fan_out || '0';
      const instability = d.instability || '0';
      const pattern = d.pattern || '';

      if (pattern === 'api') {
        f.summary = `Wide API surface — fan-in=${fanIn}, fan-out=${fanOut}, I=${instability}. This looks like an intentional API/dispatch hub — verify it's not accidental coupling.`;
        f.suggested_next_step = `If this is an intentional API/dispatch hub, this is fine — no action needed. Otherwise, split callers across focused interfaces.`;
      } else if (pattern === 'orch') {
        f.summary = `High dependency fan-out (${fanOut}), low fan-in (${fanIn}), I=${instability}. This module imports many peers — consider decomposing.`;
        f.suggested_next_step = 'Decompose into focused modules with fewer dependencies each.';
      } else {
        f.summary = `Dependency concentration — fan-in=${fanIn}, fan-out=${fanOut}, I=${instability}. High change-impact surface AND high dependency count.`;
        f.suggested_next_step = `Extract an interface to decouple ${f.path} from its dependents.`;
      }

      const iVal = parseFloat(instability);
      let iInterpretation: string;
      if (pattern === 'api') {
        iInterpretation = iVal < 0.1 ? 'very stable (API-like)' : 'stable (API-like)';
      } else if (pattern === 'orch') {
        iInterpretation = iVal > 0.9 ? 'very unstable (consumer-like)' : 'unstable (consumer-like)';
      } else {
        iInterpretation = 'balanced';
      }

      f.reasons = [];
      const fi = parseInt(fanIn);
      const fo = parseInt(fanOut);
      if (fi >= 10) f.reasons.push(`Fan-in: ${fanIn} modules import this (limit: 10)`);
      if (fo >= 10) f.reasons.push(`Fan-out: imports ${fanOut} modules (limit: 10)`);
      f.reasons.push(`Instability I=${instability} — ${iInterpretation}`);
      break;
    }

    case FindingKind.CodeDuplication: {
      const fileCount = d.file_count || '?';
      const fnName = d.fn_name || '?';

      f.summary = `Duplicate logic — fn \`${fnName}\` copied across ${fileCount} files, AI edits will not propagate`;

      f.reasons = [
        `Identical function body found in ${fileCount} different files — AI edits will not propagate`,
      ];

      f.suggested_next_step =
        'DRY it up: extract duplicated logic into a shared utility function';
      break;
    }

    case FindingKind.ReexportEntropy: {
      const totalItems = d.total_items || '?';
      const reexportCount = d.reexport_count || '?';
      const hasWildcard = d.has_wildcard === 'true';

      f.summary = `Re-export chain — ${totalItems} symbols exposed, agents traverse multiple files to resolve each import`;

      f.reasons = [
        `${reexportCount} re-export declarations`,
        `${totalItems} total re-exported symbols`,
      ];
      if (hasWildcard) {
        f.reasons.push('Contains wildcard re-exports — hides the actual public API surface');
      }

      f.suggested_next_step =
        'Flatten the re-export chain — barrel files degrade LLM path resolution';
      break;
    }
  }
}

export function enrichFindings(findings: Finding[]): void {
  for (const f of findings) {
    enrichFinding(f);
  }
}
