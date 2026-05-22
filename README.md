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

## Research Foundations

AI coding agents process your codebase through a limited context window. Research shows that **what's in that window — and what shouldn't be — directly determines cost, accuracy, and agent success rate**.

### Why context hygiene matters

| Finding | Source |
|---------|--------|
| LLM attention degrades significantly on mid-context content — model performance drops ~20% when key information isn't at the start or end of the input | [Liu et al. (2023)](https://arxiv.org/abs/2307.03172) "Lost in the Middle" |
| Adding irrelevant context to a coding task reduces LLM performance **exponentially** with context length — noise in the prompt compounds across subtasks | [Wolf et al. (2024)](https://arxiv.org/abs/2409.18028) "Compositional Hardness of Code in LLMs" |
| Real agent trajectories contain **30–60% waste tokens** — cache files, redundant tool output, and expired context accumulate to ~1M tokens per GitHub issue | [AgentDiet (2025)](https://arxiv.org/abs/2509.23586) "Improving LLM Agent Efficiency through Trajectory Reduction" |
| Removing unnecessary tokens from code before feeding to LLMs **cuts API costs by 24%** with no quality loss; simpler masking often beats LLM summarization (52% cost reduction) | [Wang et al. (2024)](https://dl.acm.org/doi/10.1145/3643753) "SlimCode" (FSE 2024); [JetBrains Research (2025)](https://blog.jetbrains.com/research/2025/12/efficient-context-management/) |
| In a study of 82,845 real developer-LLM interactions, Python code had **import errors in 20.8%** of generated snippets and undefined variables in 83.4% — unused imports and dead references are both a token drain and an error source | [Zhong et al. (2025)](https://arxiv.org/abs/2509.10402) "Developer-LLM Conversations" |

### How repo-gc applies this research

Each heuristic targets a specific, research-validated source of AI context waste:

| Heuristic | Token-waste pattern detected | Why it drains AI context |
|-----------|------------------------------|--------------------------|
| **context-bomb** | Files exceeding token thresholds (500+ lines) | Oversized files crowd out task-relevant content mid-context, where [Liu et al.](https://arxiv.org/abs/2307.03172) show attention is weakest |
| **dead-weight** | Modules with zero fan-in, never imported by any other file | Unreferenced code occupies context window slots with zero task utility — exactly the "useless information" category [AgentDiet](https://arxiv.org/abs/2509.23586) identifies as 30–60% of agent context |
| **reexport-entropy** | Deep re-export chains, wildcard barrels, init-file re-export cascades | Each indirection layer forces the LLM to resolve symbols across multiple files — [Wolf et al.](https://arxiv.org/abs/2409.18028) show compositional indirection grows exponentially harder for LLMs |
| **coupling-hotspot** | High fan-in/fan-out modules (god/api/orch patterns) via instability index | Highly coupled modules force the agent to hold more dependency state in working context; irrelevant context in one subtask leaks into others via shared dependencies |
| **code-duplication** | Type 2 clones — functions with identical normalized AST bodies across ≥2 files | [Li et al. (2024)](https://arxiv.org/abs/2411.06638) find LLMs struggle to generalize edits across semantically identical code; duplicated logic means a bug fix in one location is likely missed in its clones |
| **unused-import** | Imported symbols never referenced in the file's identifier set | Dead imports inflate token count with zero semantic value; [SlimCode](https://dl.acm.org/doi/10.1145/3643753) (FSE 2024) shows that removing low-impact tokens reduces API costs without quality loss |

**Methodology summary:** repo-gc statically analyzes your codebase to measure these 6 token-waste vectors, computes a composite AI Friction Score (0–100), and produces a ranked list of findings — so you can fix the patterns that matter most **before** they reach the LLM context window.

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
