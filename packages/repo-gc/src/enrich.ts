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
      const tokens = d.estimated_tokens || '?';
      const modulePath = d.module_path || f.path;

      f.summary = `Dead module — ~${lines} lines / ~${tokens} tokens loaded into agent context but never referenced`;

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

    case FindingKind.ErrorSwallow: {
      const count = d.empty_catch_count || '?';
      const preview = d.preview || '';

      f.summary = `Empty error handler — ${count} catch/except block(s) silently swallow errors: ${preview}`;

      f.reasons = [
        'Silently discards errors — callers cannot know an error occurred',
        'AI agents cannot reason about failure modes hidden by empty handlers',
      ];

      f.suggested_next_step =
        'At minimum log the error; better: handle it or propagate it to the caller';
      break;
    }

    case FindingKind.BranchDensity: {
      const branchCount = d.branch_count || '?';
      const fnCount = d.function_count || '?';
      const avg = d.avg_branches_per_fn || '?';

      f.summary = `High branch density — avg ${avg} branches per function across ${fnCount} functions. Complex control flow degrades LLM reasoning (RE2-Bench: 51.5% perf drop).`;

      f.reasons = [
        `${branchCount} total branches across ${fnCount} functions (avg ${avg} per function)`,
        'High cyclomatic complexity correlates with significant LLM performance degradation',
      ];

      f.suggested_next_step =
        'Decompose complex functions: extract conditionals into smaller helper functions, use early returns, and reduce nesting';
      break;
    }

    case FindingKind.DeepNesting: {
      const maxDepth = d.max_depth || '?';
      const limit = d.limit || '?';

      f.summary = `Deep nesting — max depth ${maxDepth} levels. Deeply nested code increases cognitive load for LLMs; they skip condition bodies and miss deeply nested blocks.`;

      f.reasons = [
        `Maximum nesting depth of ${maxDepth} exceeds limit of ${limit}`,
        'Deeply nested code degrades LLM reasoning; RE2-Bench shows significant performance drops with deep nesting',
      ];

      f.suggested_next_step =
        'Flatten nested blocks: use early returns, guard clauses, extract helper functions, or invert conditions';
      break;
    }

    case FindingKind.ImportDiversity: {
      const domainCount = d.domain_count || '?';
      const domains = d.domains || '?';
      const limit = d.limit || '?';

      f.summary = `God file — imports from ${domainCount} different domains (limit: ${limit}). Files touching many unrelated concerns waste AI context and degrade reasoning (Program Decomposition, arXiv 2401.12412).`;

      f.reasons = [
        `${domainCount} distinct import domains (limit: ${limit})`,
        `Domains: ${domains}`,
        'Reducing cross-file dependencies shrinks context to ~5% of window',
      ];

      f.suggested_next_step =
        'Split into focused modules grouped by domain; each module should import from related concerns only';
      break;
    }

    case FindingKind.DangerousPattern: {
      const count = d.dangerous_pattern_count || '?';
      const limit = d.limit || '?';
      const preview = d.preview || '';

      f.summary = `Dangerous patterns — ${count} instances of eval, unsafe, unwrap, or any-typed code (limit: ${limit})`;

      f.reasons = [
        `${count} dangerous pattern instances found`,
        'These broaden the state space LLMs must reason about',
      ];

      f.suggested_next_step =
        'Replace dangerous patterns with safer alternatives where possible (e.g., type-safe error handling instead of unwrap, concrete types instead of any)';
      break;
    }

    case FindingKind.CommentRatio: {
      const cl = d.comment_lines || '?';
      const tl = d.total_lines || '?';
      const ratio = d.ratio || '?';
      const dir = d.direction || '';

      if (dir === 'sparse') {
        f.summary = `Sparse comments — ${cl} comment lines out of ${tl} total (ratio ${ratio}). LLMs struggle to infer intent from uncommented code.`;
        f.reasons = [
          `Comment ratio ${ratio} is below minimum threshold`,
          'Insufficient comments make it harder for AI agents to understand intent and edge cases',
        ];
        f.suggested_next_step = 'Add doc comments to public APIs and explanatory comments for non-obvious logic';
      } else {
        f.summary = `Verbose comments — ${cl} comment lines out of ${tl} total (ratio ${ratio}). Excessive comments waste context window tokens.`;
        f.reasons = [
          `Comment ratio ${ratio} exceeds maximum threshold`,
          'Excessive comments consume valuable context window space',
        ];
        f.suggested_next_step = 'Trim redundant or obvious comments; let well-named code speak for itself';
      }
      break;
    }

    case FindingKind.StringlyTyped: {
      const count = d.string_comparison_count || '?';
      const limit = d.limit || '?';
      const preview = d.preview || '';

      f.summary = `Stringly-typed — ${count} string literal comparisons (limit: ${limit}). Magic strings in control flow bypass enum safety.`;

      f.reasons = [
        `${count} string comparisons found (limit: ${limit})`,
        'Stringly-typed code makes it harder for LLMs to reason about valid states',
      ];

      f.suggested_next_step =
        'Replace magic string comparisons with typed enums or constants for better LLM reasoning and type safety';
      break;
    }

    case FindingKind.NamingEntropy: {
      const conventionCounts = d.convention_counts || '{}';
      const dominant = d.dominant_convention || '?';
      const mixedCount = d.mixed_count || '?';

      f.summary = `Mixed naming conventions — ${mixedCount} conventions in significant use across identifiers. Dominant: ${dominant}. Inconsistent naming degrades LLM reasoning about code semantics.`;

      f.reasons = [
        `${mixedCount} naming conventions coexist above 5% threshold`,
        `Dominant convention: ${dominant}`,
        `Convention distribution: ${conventionCounts}`,
      ];

      f.suggested_next_step =
        'Adopt a single naming convention (preferably consistent with language idioms) across the file to reduce LLM confusion';
      break;
    }

    case FindingKind.TypeComplexity: {
      const depth = d.max_type_depth || '?';
      const limit = d.limit || '?';

      f.summary = `Complex types — max nesting depth ${depth} levels (limit: ${limit}). Deeply nested generic types cause LLMs to lose track of type relationships.`;

      f.reasons = [
        `Maximum type nesting depth of ${depth} exceeds limit of ${limit}`,
        'Deeply nested generic types degrade LLM reasoning about type relationships',
      ];

      f.suggested_next_step =
        'Simplify complex types: use type aliases, extract type parameters, or flatten generics';
      break;
    }

    case FindingKind.ImplicitControl: {
      const dc = d.decorator_count || '?';
      const fc = d.function_count || '?';
      const ratio = d.ratio || '?';
      const limit = d.limit || '?';

      f.summary = `Hidden control flow — ${dc} decorators across ${fc} functions (ratio ${ratio}, limit ${limit}). Decorators create implicit behavior invisible to static analysis.`;

      f.reasons = [
        `${dc} decorators across ${fc} functions (ratio ${ratio})`,
        'Decorator/middleware patterns create hidden execution paths that LLMs may miss',
      ];

      f.suggested_next_step =
        'Minimize decorator nesting; document decorator side effects for AI readability';
      break;
    }
  }
}

export function enrichFindings(findings: Finding[]): void {
  for (const f of findings) {
    enrichFinding(f);
  }
}
