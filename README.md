# repo-gc

**AI-era repository hygiene analyzers — find the patterns that waste tokens, confuse AI agents, and slow down every edit. One runner, multiple language plugins.**

[![npm](https://img.shields.io/npm/v/repo-gc)](https://www.npmjs.com/package/repo-gc)
[![npm](https://img.shields.io/npm/v/repo-gc-typescript)](https://www.npmjs.com/package/repo-gc-typescript)
[![npm](https://img.shields.io/npm/v/repo-gc-python)](https://www.npmjs.com/package/repo-gc-python)
[![License](https://img.shields.io/badge/license-FSL--1.1--MIT-blue)](./LICENSE)

```bash
npx repo-gc scan
```

Zero friction. No signup. No auth. Internet not required at scan time.

---

## Quick Start

```bash
# Scan the current directory — auto-detects all languages
npx repo-gc scan

# Scan a specific language only
npx repo-gc scan --lang typescript

# Shortcuts for language-specific scans
npx repo-gc-typescript scan      # same as repo-gc scan --lang typescript
npx repo-gc-rust scan            # same as repo-gc scan --lang rust
npx repo-gc-python scan          # same as repo-gc scan --lang python

# Markdown report — shareable, screenshot-ready
npx repo-gc scan --format md > REPO_HEALTH.md

# JSON output for CI pipelines
npx repo-gc scan --format json > results.json

# Token-optimized output for LLM consumption (~73% smaller)
npx repo-gc scan --format llm > findings.llm.tsv

# Stricter thresholds for deeper analysis
npx repo-gc scan --threshold strict
```

---

## Why This Exists

AI coding agents (Claude Code, Cursor, Copilot, Windsurf) read your codebase into a limited context window. Every oversized file, every dead module, every opaque re-export chain eats into that budget. The result: **higher token costs, broken edits, slower agent reasoning.**

`repo-gc` detects 6 structural patterns that most impact AI tooling:

| Pattern | Impact on AI-assisted development |
|---------|-----------------------------------|
| **context-bomb** | Over-large files exhaust the LLM context window, causing truncated reasoning |
| **dead-weight** | Unreferenced modules loaded into agent context for zero benefit |
| **reexport-entropy** | Deep re-export chains force LLMs to resolve indirect symbol paths across multiple files |
| **coupling-hotspot** | Highly-connected modules concentrate dependency risk → higher reasoning overhead |
| **code-duplication** | Identical function bodies confuse AI edits → changes don't propagate |
| **unused-import** | Dead imports inflate token usage in every context window |

All checks are **fully deterministic** — no LLM calls, no network, no hallucination risk.

---

## Architecture

```
npx repo-gc (runner)
  ├─ discovery       → finds workspaces, enumerates files
  ├─ auto-detection  → scans extensions + manifests, activates plugins
  ├─ plugins         → language-specific parsing + heuristics → Finding[]
  ├─ scoring         → computeGlobalScore (shared)
  └─ formatting      → text, json, markdown, llm (shared)
```

---

## Packages

| Package | Status | Description |
|---------|--------|-------------|
| [`repo-gc`](./packages/repo-gc) | [![npm](https://img.shields.io/npm/v/repo-gc)](https://www.npmjs.com/package/repo-gc) | Main CLI — auto-detects languages, orchestrates plugins, formats output |
| [`repo-gc-typescript`](./packages/repo-gc-typescript) | [![npm](https://img.shields.io/npm/v/repo-gc-typescript)](https://www.npmjs.com/package/repo-gc-typescript) | TypeScript/JavaScript plugin — parses with oxc, runs 6 heuristics |
| [`repo-gc-rust`](./packages/repo-gc-rust) | [![npm](https://img.shields.io/npm/v/repo-gc-rust)](https://www.npmjs.com/package/repo-gc-rust) | Rust analyzer — standalone native binary, invocable as a plugin |
| [`repo-gc-python`](./packages/repo-gc-python) | [![npm](https://img.shields.io/npm/v/repo-gc-python)](https://www.npmjs.com/package/repo-gc-python) | Python plugin — AST analysis via Python stdlib, 6 heuristics |

Each language plugin is optional — install only what you need. The main runner auto-detects which plugins are available.

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
═══ AI Context Efficiency Report ═══
Files: 22  Lines: 2549  Est. LLM tokens: ~637k

Scores
  AI Friction:        ██████░░░░  28/100
  Context Waste:      ░░░░░░░░░░  0/100
  Structural Entropy: ██████░░░░  28/100

Findings (6)

  Coupling Hotspot (1)
    [H] packages/repo-gc-rust/src/types.rs — High coupling: fan-in=13, fan-out=0

  Dead Weight (2)
    [M] src/cli.ts — 85-line file is never imported
    [M] src/old-utils.ts — 120-line file is never imported
  ...
```

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--path` | `.` | Target directory |
| `--format` | `text` | Output: `text`, `json`, `md`, `llm` |
| `--threshold` | `normal` | Sensitivity: `strict`, `normal`, `relaxed` |
| `--include-tests` | `false` | Include test files in analysis |
| `--no-color` | `false` | Plain text output |
| `--lang` | auto | Comma-separated language plugins (e.g. `typescript,rust`) |

## Requirements

- **Node.js 18+**
- Rust toolchain only needed to build the Rust analyzer from source (prebuilt binaries available via npm)

## Build from Source

```bash
git clone https://github.com/repo-gc/repo-gc.git
cd repo-gc
pnpm install
pnpm build                      # builds all TypeScript packages
cargo build --release           # builds the Rust analyzer
pnpm test                       # runs all test suites
```

## License

**FSL-1.1-MIT** (Functional Source License 1.1, MIT Future License).

`repo-gc` is free for individuals, teams, and CI pipelines — forever. You can use it, modify it, and share it. The FSL license prevents large AI companies from packaging it into competing paid offerings without contributing back.

Each version converts to MIT after 2 years. See [LICENSE](./LICENSE) for the full text.

[CHANGELOG](./CHANGELOG.md)
