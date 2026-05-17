# repo-gc Monorepo

Multi-language repository hygiene analyzer — scans codebases for structural patterns that make AI-assisted development slower and more error-prone.

## Packages

| Package | Description | Status |
|---------|-------------|--------|
| [packages/repo-gc-rust](./packages/repo-gc-rust/) | Rust analyzer — `npx repo-gc-rust` | Stable |
| [packages/repo-gc](./packages/repo-gc/) | Wrapper CLI + shared core | Coming soon |
| [packages/repo-gc-typescript](./packages/repo-gc-typescript/) | TypeScript analyzer | Coming soon |
| [packages/repo-gc-python](./packages/repo-gc-python/) | Python analyzer | Coming soon |

---

# repo-gc-rust

AI-era Rust repository hygiene analyzer. Scans Rust codebases for structural patterns that make AI-assisted development slower and more error-prone.

## Install

```bash
npx repo-gc-rust scan           # runs instantly, no Rust required
npm install -g repo-gc-rust     # install globally
```

## Usage

```bash
repo-gc scan                                    # text output, current dir
repo-gc scan --format json --path ./my-repo    # JSON for CI
repo-gc scan --format md > report.md           # Markdown report
repo-gc scan --threshold strict                # lower thresholds, more findings
repo-gc explain src/core/session.rs            # explain one file
repo-gc report --format md > report.md        # summary report (alias for scan --format md)
repo-gc json --path ./my-repo                  # JSON-only subcommand
```

## What It Detects

| Finding | Description |
|---------|-------------|
| **context-bomb** | Files too large for efficient AI-assisted editing |
| **dead-weight** | Files not referenced by any module or use declaration |
| **reexport-entropy** | Deep `pub use` chains hiding real structure |
| **coupling-hotspot** | Modules with high import fan-in or fan-out |
| **code-duplication** | Identical function bodies copied across multiple files |
| **unused-import** | Imported names not referenced in the file body |

All checks are fully deterministic — no LLM or network calls required.

## Global Scores (0–100, higher = worse)

- **AI Hostility Score** — Overall repository friction (all findings)
- **Context Waste Score** — Token budget wasted by oversized files
- **Entropy Score** — Structural noise: dead weight, coupling, re-exports, duplication, unused imports

## Flags

| Flag | Default | Subcommands |
|------|---------|-------------|
| `--path` | `.` | scan, report, json, explain |
| `--format` | `text` | scan, report (`text`/`json`/`md`) |
| `--threshold` | `normal` | scan, report, json (`strict`/`normal`/`relaxed`) |
| `--include-tests` | `false` | scan, report, json |
| `--no-color` | `false` | scan (text format) |

## Requirements

- Node.js 14+
- No internet connection required at scan time (binary downloaded once at install)

## License

MIT
