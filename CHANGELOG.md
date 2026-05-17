# Changelog

## 0.1.0 (2026-05-17)

Initial public release.

### Heuristics (6 categories)
- **context-bomb** — files over 500 lines that overload LLM context windows
- **dead-weight** — unreferenced modules loaded into agent context for no benefit
- **reexport-entropy** — deep `pub use` chains forcing LLMs to resolve indirections
- **coupling-hotspot** — modules with high import fan-in or fan-out increasing reasoning overhead
- **code-duplication** — identical function bodies across files causing AI edit inconsistency
- **unused-import** — dead imports inflating token usage

### Global Scores
- **AI Friction Score** — composite of all findings (0-100)
- **Context Waste Score** — token budget wasted by oversized files (0-100)
- **Structural Entropy Score** — noise from coupling, dead code, re-exports, duplication (0-100)

### Output Formats
- Text (terminal with color)
- JSON (for CI pipelines)
- Markdown (shareable reports)

### Distribution
- Available via `npx repo-gc-rust scan`
- Prebuilt binaries for macOS (x86_64, ARM), Linux (x86_64, ARM), Windows (x86_64)
- No auth, no signup, no internet required at scan time
