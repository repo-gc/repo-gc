# repo-gc

**AI-era repository hygiene analyzers — find the patterns that waste tokens, confuse AI agents, and slow down every edit. One vision, multiple languages.**

```bash
npx repo-gc-rust scan     # Rust (stable)
npx repo-gc-typescript scan  # TypeScript (coming soon)
npx repo-gc-python scan      # Python (coming soon)
```

Zero friction. No signup. No auth. Internet not required at scan time.

---

## Packages

| Package | Status | Description |
|---------|--------|-------------|
| [`repo-gc-rust`](./packages/repo-gc-rust) | **Stable** | Rust analyzer — detects oversized files, dead modules, re-export entropy, coupling hotspots, duplication, zombie imports |
| `repo-gc-typescript` | Planned | TypeScript analyzer — stub only |
| `repo-gc-python` | Planned | Python analyzer — stub only |
| `repo-gc` | Planned | Umbrella meta-package — runs all language analyzers in one pass |

---

## Why This Exists

AI coding agents (Claude Code, Cursor, Copilot, Windsurf) read your codebase into a limited context window. Every oversized file, every dead module, every opaque re-export chain eats into that budget. The result: **higher token costs, broken edits, slower agent reasoning.**

`repo-gc-rust` detects 6 structural patterns that most impact AI tooling:

| Pattern | Impact on AI-assisted development |
|---------|-----------------------------------|
| **context-bomb** | Over-large files exhaust the LLM context window, causing truncated reasoning |
| **dead-weight** | Unreferenced modules loaded into agent context for zero benefit |
| **reexport-entropy** | Deep `pub use` chains force LLMs to resolve indirect symbol paths across multiple files |
| **coupling-hotspot** | Highly-connected modules concentrate dependency risk → higher reasoning overhead |
| **code-duplication** | Identical function bodies confuse AI edits → changes don't propagate |
| **unused-import** | Dead imports inflate token usage in every context window |

All checks are **fully deterministic** — no LLM calls, no network, no hallucination risk.

---

## Quick Start

```bash
# Scan the current directory (text output)
npx repo-gc-rust scan

# Markdown report — screenshot-ready, shareable
npx repo-gc-rust scan --format md > REPO_HEALTH.md

# Token-optimized output for LLM consumption
npx repo-gc-rust scan --format llm > findings.llm.tsv

# JSON output for CI pipelines
npx repo-gc-rust scan --format json > results.json

# Explain why a specific file is flagged
npx repo-gc-rust explain src/main.rs

# Stricter thresholds for deeper analysis
npx repo-gc-rust scan --threshold strict
```

> **Note:** The npm package is `repo-gc-rust`; the binary inside it is `repo-gc`. `npx` handles this automatically.

## Understanding Your Scores (0–100)

| Score | 0–19 | 20–39 | 40–69 | 70–100 |
|-------|------|-------|-------|--------|
| **AI Friction** | AI-friendly | Moderate friction | Concerning | Critical |
| **Context Waste** | Efficient | Some waste | Significant | Severe |
| **Structural Entropy** | Clean | Moderate noise | Complex | Chaotic |

- **AI Friction** — Composite: how hard your codebase is for AI agents to work with effectively
- **Context Waste** — Token budget burned by oversized files
- **Structural Entropy** — Noise from dead code, coupling, re-exports, duplication, unused imports

## Example Output

```
═══ repo-gc — AI Context Efficiency ═══
Files: 22  Lines: 2549  Est. LLM tokens: ~637k

Global Scores (0-100, higher = worse)
  AI Friction:        ██████░░░░  28/100  (LOW)
  Context Waste:      ░░░░░░░░░░  0/100   (LOW)
  Structural Entropy: ██████░░░░  28/100  (LOW)

▸  Your repo wastes ~3% of agent context capacity (~0.3x Claude sessions)

Detected Patterns (6 total)

  1. [MEDIUM] packages/repo-gc-rust/src/types.rs — coupling-hotspot
     • Fan-in: 13 modules import this (limit: 10)

  2. [LOW] packages/repo-gc-rust/src/cli.rs — unused-import
     • 3 imported names not referenced in file body; macros/derives may cause false positives
  ...
```

## Output Formats

| Format | Flag | Use case |
|--------|------|----------|
| Text | `--format text` (default) | Terminal, human-readable with ASCII score bars |
| Markdown | `--format md` | Shareable reports, screenshots, PR comments |
| JSON | `--format json` | CI integration, programmatic consumption |
| LLM | `--format llm` | Token-optimized TSV for AI agent consumption (~73% smaller) |

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--path` | `.` | Target directory |
| `--format` | `text` | Output: `text`, `json`, `md`, `llm` |
| `--threshold` | `normal` | Sensitivity: `strict`, `normal`, `relaxed` |
| `--include-tests` | `false` | Include test files in analysis |
| `--no-color` | `false` | Plain text output |

## How It Works (Rust analyzer)

1. **Discovers** your Cargo workspace via `cargo_metadata`
2. **Parses** each `.rs` file with `syn` (Rust's official parser)
3. **Builds** an import dependency graph (fan-in / fan-out analysis)
4. **Runs** 6 heuristic analyzers over the parsed AST in parallel (`rayon`)
5. **Computes** three global scores normalized to 0–100
6. **Reports** findings sorted by severity with suggested remediation

All analysis is **fully offline and deterministic** — no external API calls, no data leaves your machine.

## Install

```bash
npx repo-gc-rust scan           # zero-install, runs instantly
cargo install repo-gc-rust      # Rust toolchain required
```

Prebuilt binaries for macOS (x86_64, ARM), Linux (x86_64, ARM, glibc ≥ 2.31), Windows (x86_64).

## Requirements

- **Node.js 14+** (for `npx` / npm install)
- No Rust toolchain needed at runtime
- No internet at scan time (binary downloaded once at install)

## License

**FSL-1.1-MIT** (Functional Source License 1.1, MIT Future License).

`repo-gc` is free for individuals, teams, and CI pipelines — forever. You can use it, modify it, and share it. The FSL license prevents large AI companies from packaging it into competing paid offerings without contributing back.

Each version converts to MIT after 2 years. See [LICENSE](./LICENSE) for the full text.

## What's Next

This is the **free, zero-friction scanner** — the distribution engine. Paid features (CI integration, trend tracking, team dashboards, PR automation) are in development. The scanner will always be free.

Future language analyzers (TypeScript, Python) will follow the same architecture: deterministic AST parsing → dependency graph → heuristic scoring → multi-format output.

---

[CHANGELOG](./CHANGELOG.md)
