# Changelog

## 0.2.8 (2026-05-23)

### New Heuristics
- **10 new code-quality heuristics** across all language backends (TypeScript, Rust, Python):
  - **god-module** — modules that do too many unrelated things, increasing LLM reasoning burden
  - **api-violation** — public API surface that breaks encapsulation conventions
  - **orchestrator** — modules that excessively delegate, making control flow hard to trace
  - **interface-bloat** — interfaces/protocols with too many members, raising cognitive load
  - **deep-nesting** — deeply nested control flow that strains LLM context tracking
  - **magic-number** — unnamed literals that force LLMs to guess semantics
  - **long-parameter-list** — functions with too many parameters, complicating call-site reasoning
  - **side-effect** — functions with hidden side effects that mislead AI refactoring
  - **exception-masking** — caught-and-swallowed errors that hide failure paths from AI analysis
  - **todo-bomb** — accumulated TODO/FIXME comments signaling deferred decisions

### Architecture
- **Canonical config**: Centralized heuristic definitions (labels, thresholds, scores) in a single source of truth, eliminating per-plugin duplication
- **repo-gc-shared removed**: Fully DRY'd the heuristic layer — all 10 new heuristics share one definition across backends
- Consistent package descriptions and repository URLs across all `repo-gc-*` packages

### Fixes
- Completed scaffolding for new heuristics — missing labels, thresholds, and scores now wired through

## 0.2.7 (2026-05-22)

### Shared Heuristics Layer
- **runHeuristics()**: Shared language-agnostic function that runs all 6 heuristics, deduplicating plugin boilerplate across TypeScript, Rust, and Python
- **Type 2 clone detection**: Added Type 2 (identifier-normalized) code duplication detection alongside existing Type 1 (exact-match) detection

### Architecture
- Inlined `repo-gc-shared` into `repo-gc` — simplified the plugin dependency graph, eliminating an unnecessary package
- Python plugin made public (repo-gc-python now published to npm)

### Fixes
- Fixed wildcard re-export false-negative where `*` re-exports in `__init__.py` were not being detected
- Updated evidence keys for consistency across all heuristics
- Build config fixes

## 0.2.5 (2026-05-20)

### Python Plugin
- **repo-gc-python**: Python hygiene analyzer with 6 heuristics
- Spawns `python -m repo_gc_python` as a subprocess for AST analysis
- CLI shortcut: `npx repo-gc-python scan` (wraps `repo-gc scan --lang python`)
- Built on Python stdlib (`ast`, `pathlib`) — zero Python dependencies
- Supports Python >=3.10
- Detects: context bombs, dead weight, re-export entropy, coupling hotspots, code duplication, unused imports

## 0.2.0 (2026-05-19)

Initial public release — monorepo architecture with the main CLI runner, shared core, and TypeScript plugin.

### Architecture
- **repo-gc**: Main CLI runner — workspace discovery, language auto-detection, plugin orchestration, output formatting
- Internal shared package: types, scoring formula, thresholds, token estimation, and 4 reporters (text, JSON, markdown, LLM-TSV)
- **repo-gc-typescript**: Full TypeScript/JavaScript plugin — oxc parser, import graph, 6 heuristics
- **repo-gc-rust**: Rust analyzer — standalone native binary, invocable as a plugin via CLI delegation

### Heuristics (6 categories)
- **context-bomb** — files over 500 lines that overload LLM context windows
- **dead-weight** — unreferenced modules loaded into agent context for no benefit
- **reexport-entropy** — barrel files and re-export chains forcing LLMs to resolve indirections
- **coupling-hotspot** — modules with high import fan-in or fan-out increasing reasoning overhead
- **code-duplication** — identical function bodies across files causing AI edit inconsistency
- **unused-import** — dead imports inflating token usage

### Global Scores
- **AI Friction Score** — composite of all findings (0-100)
- **Context Waste Score** — token budget wasted by oversized files (0-100)
- **Structural Entropy Score** — noise from coupling, dead code, re-exports, duplication (0-100)

### Output Formats
- Text (terminal with color and ASCII score bars)
- JSON (for CI pipelines, schema-compatible with Rust analyzer)
- Markdown (shareable reports)
- LLM (token-optimized TSV for AI agent consumption, ~73% smaller)

### Distribution
- `npx repo-gc scan` — auto-detects languages
- `npx repo-gc-typescript scan` — TypeScript/JavaScript only
- `npx repo-gc-rust scan` — Rust only (native binary)
- `pnpm add repo-gc` — library usage, exports `scan()` function
- Zero auth, no signup, no internet required at scan time
