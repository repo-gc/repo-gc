# repo-gc-typescript

**TypeScript/JavaScript language plugin for repo-gc — detects oversized files, dead modules, coupling hotspots, barrel re-exports, duplicate code, and unused imports.**

[![npm](https://img.shields.io/npm/v/repo-gc-typescript)](https://www.npmjs.com/package/repo-gc-typescript)
[![License](https://img.shields.io/badge/license-FSL--1.1--MIT-blue)](../../LICENSE)

Uses [oxc](https://oxc.rs) for fast AST parsing and builds a module import graph to run 6 heuristics. Install alongside the main `repo-gc` runner.

```bash
npx repo-gc-typescript scan
```

## Quick Start

```bash
# Standalone (auto-wraps repo-gc with --lang typescript)
npx repo-gc-typescript scan

# Via the main runner
npx repo-gc scan --lang typescript

# Markdown report
npx repo-gc-typescript scan --format md > REPO_HEALTH.md

# JSON for CI
npx repo-gc-typescript scan --format json
```

## Install

```bash
pnpm add repo-gc repo-gc-typescript
# or: npm install repo-gc repo-gc-typescript
```

## Heuristics

| Heuristic | What it detects |
|-----------|----------------|
| **context-bomb** | Files over 500 lines that exhaust LLM context windows |
| **dead-weight** | Unreferenced modules loaded into agent context for no benefit |
| **reexport-entropy** | Barrel files and deep re-export chains that force LLMs to resolve indirections |
| **coupling-hotspot** | Modules with high import fan-in or fan-out that concentrate dependency risk |
| **code-duplication** | Identical function bodies across files that cause AI edit inconsistency |
| **unused-import** | Dead imports that inflate token usage in every context window |

All checks are deterministic — no LLM calls, no network, no hallucination risk.

## API

```ts
import { typescriptPlugin } from 'repo-gc-typescript';

// typescriptPlugin implements the LanguagePlugin interface:
// { name, displayName, fileExtensions, testFilePatterns, requiredManifests, analyzeLanguage }

const result = typescriptPlugin.analyzeLanguage(files, workspaceRoot, thresholds);
// → { findings: Finding[], skipped: number, errors: string[] }
```

## Parser

Parses TypeScript/JavaScript with [oxc-parser](https://www.npmjs.com/package/oxc-parser) — a Rust-based parser that's significantly faster than Babel or TypeScript's compiler API. Module resolution uses [unrs-resolver](https://www.npmjs.com/package/unrs-resolver) for accurate import graph construction.

## Requirements

- Node.js 18+
- `repo-gc` (peer dependency — provides shared types, scoring, and formatting)

## License

FSL-1.1-MIT — free for individuals, teams, and CI pipelines. See [LICENSE](../../LICENSE).
