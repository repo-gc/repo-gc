# repo-gc

**AI-era repository hygiene analyzers — find the patterns that waste tokens, confuse AI agents, and slow down every edit. One runner, multiple language plugins.**

[![npm](https://img.shields.io/npm/v/repo-gc)](https://www.npmjs.com/package/repo-gc)
[![npm](https://img.shields.io/npm/v/repo-gc-typescript)](https://www.npmjs.com/package/repo-gc-typescript)
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

# Markdown report — shareable, screenshot-ready
npx repo-gc scan --format md > REPO_HEALTH.md

# JSON output for CI pipelines
npx repo-gc scan --format json > results.json

# Token-optimized output for LLM consumption (~73% smaller)
npx repo-gc scan --format llm > findings.llm.tsv

# Stricter thresholds for deeper analysis
npx repo-gc scan --threshold strict
```

The main runner auto-detects which languages are present — no `--lang` flag needed.

> **Note:** The Rust package (`repo-gc-rust`) is a prebuilt native binary. The main runner shells out to it for Rust analysis.

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

Each language package is a **plugin** — it exports a single function `analyzeLanguage(files, root, thresholds) → { findings, skipped, errors }`. The runner handles everything else: discovery, ID assignment, scoring, and rendering. Adding a new language is ~200 lines of code.

---

## Packages

| Package | Status | Description |
|---------|--------|-------------|
| [`repo-gc`](./packages/repo-gc) | [![npm](https://img.shields.io/npm/v/repo-gc)](https://www.npmjs.com/package/repo-gc) | Main CLI — auto-detects languages, orchestrates plugins, formats output |
| [`repo-gc-typescript`](./packages/repo-gc-typescript) | [![npm](https://img.shields.io/npm/v/repo-gc-typescript)](https://www.npmjs.com/package/repo-gc-typescript) | TypeScript/JavaScript plugin — parses with oxc, runs 6 heuristics |
| [`repo-gc-rust`](./packages/repo-gc-rust) | [![npm](https://img.shields.io/npm/v/repo-gc-rust)](https://www.npmjs.com/package/repo-gc-rust) | Rust analyzer — standalone native binary, invocable as a plugin |
| `repo-gc-python` | — | Python plugin — coming soon |

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
| `--lang` | auto | Comma-separated language plugins (e.g. `typescript,rust`) |

## How It Works

1. **Discovers** your workspace via `package.json` workspaces / `pnpm-workspace.yaml`
2. **Detects** which languages are present (file extensions + manifest files)
3. **Dispatches** to language plugins — each parses with its native parser (oxc for TS/JS, syn for Rust)
4. **Runs** 6 heuristic analyzers per language over the parsed AST
5. **Computes** global scores normalized to 0–100 using the shared scoring formula
6. **Formats** findings in your choice of text, JSON, Markdown, or LLM-optimized TSV

All analysis is **fully offline and deterministic** — no external API calls, no data leaves your machine.

## Install

### Quick run (zero-install)

```bash
npx repo-gc scan                 # auto-detects languages
npx repo-gc-typescript scan      # TypeScript/JavaScript only
npx repo-gc-rust scan            # Rust only (native binary)
```

### Install as a dependency

```bash
pnpm add repo-gc repo-gc-typescript
# or
npm install repo-gc repo-gc-typescript
```

```ts
import { scan } from 'repo-gc';
const report = await scan({ path: '.', format: 'json', threshold: 'normal', includeTests: false, color: false }, []);
console.log(report);
```

## Requirements

- **Node.js 18+**
- No Rust toolchain needed for TypeScript/JavaScript analysis
- Rust toolchain required to build the Rust analyzer from source (prebuilt binaries available via npm)

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

## What's Next

Future language plugins (Python, Go) follow the same architecture: implement `analyzeLanguage(files, root, thresholds) → { findings, skipped, errors }` and the runner handles the rest. Each plugin is ~200 lines of code.

---

[CHANGELOG](./CHANGELOG.md)
