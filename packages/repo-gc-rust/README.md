# repo-gc-rust

**Rust repository hygiene analyzer — finds patterns that waste tokens, confuse AI agents, and break AI-assisted edits.**

[![Crates.io](https://img.shields.io/crates/v/repo-gc-rust)](https://crates.io/crates/repo-gc-rust)
[![npm](https://img.shields.io/npm/v/repo-gc-rust)](https://www.npmjs.com/package/repo-gc-rust)
[![License](https://img.shields.io/badge/license-FSL--1.1--MIT-blue)](../../LICENSE)

A standalone native binary that parses Rust code with [syn](https://docs.rs/syn), builds an import graph, and runs 6 heuristics. Fast — uses [rayon](https://docs.rs/rayon) for parallel parsing. Can run standalone or as a plugin to the main `repo-gc` runner.

```bash
npx repo-gc-rust scan
```

## Quick Start

```bash
# Standalone (runs the native binary directly)
npx repo-gc-rust scan

# Via the main runner
npx repo-gc scan --lang rust

# Markdown report
npx repo-gc-rust scan --format md > REPO_HEALTH.md

# JSON for CI
npx repo-gc-rust scan --format json

# Token-optimized output for LLM consumption
npx repo-gc-rust scan --format llm
```

## Install

### Prebuilt binary (npm)

```bash
npm install -g repo-gc-rust
repo-gc-rust scan
```

### From source (cargo)

```bash
cargo install repo-gc-rust
repo-gc-rust scan
```

## Commands

| Command | Description |
|---------|-------------|
| `scan` | Scan for AI-context-wasting patterns |
| `report` | Generate an AI Context Efficiency report |
| `json` | Emit findings as JSON (for CI integration) |
| `explain <file>` | Explain why a file degrades AI context efficiency |

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--path` | `.` | Target directory |
| `--format` | `text` | Output: `text`, `json`, `md`, `llm` |
| `--threshold` | `normal` | Sensitivity: `strict`, `normal`, `relaxed` |
| `--include-tests` | `false` | Include test files in analysis |
| `--no-color` | — | Plain text output |

### Threshold Levels

| Level | Line limit | Fan-in limit | Fan-out limit | Re-export limit |
|-------|-----------|-------------|--------------|----------------|
| `strict` | 300 | 5 | 10 | 5 |
| `normal` | 500 | 10 | 15 | 10 |
| `relaxed` | 1000 | 20 | 25 | 20 |

## Example Output

```
═══ AI Context Efficiency Report ═══
Files: 47  Lines: 12400  Est. LLM tokens: ~3100k

Scores
  AI Friction:        ██████░░░░  28/100
  Context Waste:      ░░░░░░░░░░  0/100
  Structural Entropy: ██████░░░░  28/100

Findings (6)

  Coupling Hotspot (1)
    [H] src/types.rs — High coupling: fan-in=13, fan-out=0

  Dead Weight (2)
    [M] src/cli.rs — 85-line file is never imported
    [M] src/old-utils.rs — 120-line file is never imported
  ...
```

## Architecture

Parses Rust source with `syn`, builds a module-level import graph, then runs 6 heuristics:

| Heuristic | What it detects |
|-----------|----------------|
| **context-bomb** | Files exceeding line count thresholds |
| **dead-weight** | Orphaned modules not imported by anything |
| **reexport-entropy** | Files with excessive `pub use` re-exports |
| **coupling-hotspot** | Modules with high fan-in or fan-out |
| **code-duplication** | Identical function bodies across files |
| **unused-import** | Unused `use` statements |

Analysis runs in parallel via `rayon`. Output formats: text (terminal with ASCII score bars), JSON, Markdown, and LLM-optimized TSV.

## Requirements

- Rust toolchain (to build from source) or prebuilt binary via npm
- No runtime dependencies — single static binary

## License

FSL-1.1-MIT — free for individuals, teams, and CI pipelines. See [LICENSE](../../LICENSE).
