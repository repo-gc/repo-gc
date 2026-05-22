# repo-gc

**Multi-language repository hygiene analyzer — find patterns that waste tokens, confuse AI agents, and slow down every edit.**

[![npm](https://img.shields.io/npm/v/repo-gc)](https://www.npmjs.com/package/repo-gc)
[![License](https://img.shields.io/badge/license-FSL--1.1--MIT-blue)](https://github.com/repo-gc/repo-gc/blob/main/LICENSE)

`repo-gc` is the main CLI runner. It discovers your workspace, auto-detects languages, dispatches to language plugins, and formats the output. Install it along with the language plugins you need.

```bash
npx repo-gc scan
```

## Quick Start

```bash
# Auto-detect languages and scan
npx repo-gc scan

# Scan TypeScript only
npx repo-gc scan --lang typescript

# Markdown report
npx repo-gc scan --format md > REPO_HEALTH.md

# JSON for CI pipelines
npx repo-gc scan --format json

# Token-optimized output for LLM consumption (~73% smaller)
npx repo-gc scan --format llm
```

## Install

```bash
pnpm add repo-gc repo-gc-typescript
# or: npm install repo-gc repo-gc-typescript
```

Each language plugin is optional — install only what you need. The runner auto-detects which plugins are available.

## Commands

| Command | Description |
|---------|-------------|
| `scan` | Scan the repository for AI-context-wasting patterns |
| `report` | Generate an AI Context Efficiency report (alias for scan) |
| `json` | Emit findings as JSON (shortcut for `scan --format json`) |
| `explain <file>` | Explain why a specific file degrades AI context efficiency |

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--path` | `.` | Target directory |
| `--format` | `text` | Output: `text`, `json`, `md`, `llm` |
| `--threshold` | `normal` | Sensitivity: `strict`, `normal`, `relaxed` |
| `--include-tests` | `false` | Include test files in analysis |
| `--no-color` | `false` | Plain text output |
| `--lang` | auto | Comma-separated language plugins (e.g. `typescript,rust`) |

## API

```ts
import { scan } from 'repo-gc';

const report = await scan({
  path: '.',
  format: 'json',
  threshold: { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 },
  includeTests: false,
  color: false,
  version: '1.0.0',
}, []); // plugins array — auto-loaded in CLI, pass manually for programmatic use

console.log(report); // string in requested format
```

### Plugin Interface

Each language package exports a `LanguagePlugin`:

```ts
interface LanguagePlugin {
  name: string;
  displayName: string;
  fileExtensions: string[];
  testFilePatterns: RegExp[];
  requiredManifests: string[];
  analyzeLanguage(files: SourceFile[], root: string, thresholds: Thresholds): AnalysisResult | Promise<AnalysisResult>;
}
```

## Language Plugins

| Plugin | Package | Coverage |
|--------|---------|----------|
| TypeScript/JavaScript | [`repo-gc-typescript`](https://github.com/repo-gc/repo-gc/tree/main/packages/repo-gc-typescript) | `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs` |
| Rust | [`repo-gc-rust`](https://github.com/repo-gc/repo-gc/tree/main/packages/repo-gc-rust) | `.rs` (native binary, invoked as subprocess) |
| Python | [`repo-gc-python`](https://github.com/repo-gc/repo-gc/tree/main/packages/repo-gc-python) | `.py` (spawns `python -m repo_gc_python`) |

## Requirements

- Node.js 18+
- Language plugins installed separately (each brings its own parser)

## License

FSL-1.1-MIT — free for individuals, teams, and CI pipelines. See [LICENSE](https://github.com/repo-gc/repo-gc/blob/main/LICENSE).
