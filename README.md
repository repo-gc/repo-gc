# repo-gc-rust

AI-era Rust repository hygiene analyzer. Scans Rust codebases for structural patterns that make AI-assisted development slower and more error-prone.

> **Requires Rust/Cargo** — compiles a native binary on first install. Install from [rustup.rs](https://rustup.rs/).

## Install

```bash
npx repo-gc-rust scan           # builds once (~30s), then runs
npm install -g repo-gc-rust     # install globally
```

## Usage

```bash
repo-gc scan                                    # text output, current dir
repo-gc scan --format json --path ./my-repo    # JSON for CI
repo-gc scan --format md > report.md           # Markdown report
repo-gc scan --threshold strict                # lower thresholds, more findings
repo-gc explain src/core/session.rs            # explain one file
```

## What It Detects

| Finding | Description |
|---------|-------------|
| **context-bomb** | Files too large for efficient AI-assisted editing |
| **dead-weight** | Files not referenced by any module or use declaration |
| **reexport-entropy** | Deep `pub use` chains hiding real structure |
| **coupling-hotspot** | Modules with high import fan-in or fan-out |

## Global Scores (0–100, higher = worse)

- **AI Hostility Score** — Overall repository friction
- **Context Waste Score** — Waste from oversized files
- **Entropy Score** — Structural noise (dead weight, coupling, re-exports)

## Flags

| Flag | Default | Subcommands |
|------|---------|-------------|
| `--path` | `.` | scan, report, json, explain |
| `--format` | `text` | scan, report (`text`/`json`/`md`) |
| `--threshold` | `normal` | scan, report, json (`strict`/`normal`/`relaxed`) |
| `--include-tests` | `false` | scan, report, json |
| `--no-color` | `false` | scan (text format) |

## Requirements

- Rust 1.70+ with Cargo ([rustup.rs](https://rustup.rs/))
- Node.js 14+ (npm wrapper only)
- No internet connection at scan time

## License

MIT
