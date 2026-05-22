# repo-gc-python

**Python language plugin for repo-gc — detects oversized files, dead modules, coupling hotspots, re-export chains, duplicate code, and unused imports in Python codebases.**

[![License](https://img.shields.io/badge/license-FSL--1.1--MIT-blue)](https://github.com/repo-gc/repo-gc/blob/main/LICENSE)

Uses Python's stdlib `ast` module for parsing — zero Python dependencies. The plugin spawns `python -m repo_gc_python` as a subprocess and normalizes the JSON output.

```bash
npx repo-gc-python scan
```

## Quick Start

```bash
# Standalone (auto-wraps repo-gc with --lang python)
npx repo-gc-python scan

# Via the main runner
npx repo-gc scan --lang python

# JSON for CI
npx repo-gc-python scan --format json
```

## Install

```bash
# No install needed — just run:
npx repo-gc-python scan
```

`repo-gc-python` depends on `repo-gc`, so npx pulls in everything automatically.

## Heuristics

| Heuristic | What it detects |
|-----------|----------------|
| **context-bomb** | Files over 500 lines that exhaust LLM context windows |
| **dead-weight** | Unreferenced modules loaded into agent context for no benefit |
| **reexport-entropy** | `__init__.py` files with excessive wildcard re-exports |
| **coupling-hotspot** | Modules with high import fan-in or fan-out |
| **code-duplication** | Identical function bodies across files |
| **unused-import** | Dead imports that inflate token usage |

All checks run locally via Python's stdlib `ast` — no LLM calls, no network, no third-party Python packages.

## Architecture

The plugin is a thin JavaScript bridge that:
1. Locates the bundled Python package (`python/repo_gc_python/`)
2. Finds a `python3`/`python` binary on `PATH`
3. Spawns `python -m repo_gc_python scan --path <root> --format json --threshold <level>`
4. Normalizes the JSON findings into the shared `Finding[]` format

The actual analysis happens entirely in Python, using `ast` for parsing and `pathlib` for file discovery. Supports Python >=3.10.

## API

```ts
import { pythonPlugin } from 'repo-gc-python';

// pythonPlugin implements the LanguagePlugin interface
// Uses selfDiscovers: true — the Python side handles its own file enumeration

const result = await pythonPlugin.analyzeLanguage([], workspaceRoot, thresholds);
// → { findings: Finding[], skipped: number, errors: string[] }
```

## Requirements

- Node.js 18+
- Python >=3.10 (no extra packages needed — uses stdlib only)
- `repo-gc` (peer dependency)

## License

FSL-1.1-MIT — free for individuals, teams, and CI pipelines. See [LICENSE](https://github.com/repo-gc/repo-gc/blob/main/LICENSE).
