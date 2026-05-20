# Changelog

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
