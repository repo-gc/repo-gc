import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve, join, delimiter } from 'node:path';
import { existsSync, chmodSync } from 'node:fs';
import type { LanguagePlugin, AnalysisResult, SourceFile } from 'repo-gc-shared';
import { FindingKind, Severity, runtimeNotFound, processExited, outputParseFailed, spawnFailed } from 'repo-gc-shared';
import type { Finding, Thresholds } from 'repo-gc-shared';

const __dirname = dirname(fileURLToPath(import.meta.url));

function findBinary(): string | null {
  // __dirname is packages/repo-gc/dist/ (dev) or node_modules/repo-gc/dist/ (npm)
  const bundled = resolve(__dirname, '../bin/repo-gc');
  if (existsSync(bundled)) {
    // npm may strip the execute bit during publish; restore it
    try { chmodSync(bundled, 0o755); } catch { /* readonly fs */ }
    return bundled;
  }

  // Search PATH — the Cargo binary is named 'repo-gc' per [[bin]] in Cargo.toml
  const pathDirs = (process.env.PATH || '').split(delimiter);
  for (const dir of pathDirs) {
    const p = join(dir, 'repo-gc');
    if (p !== bundled && existsSync(p)) return p;
  }
  return null;
}

const RUST_KIND_MAP: Record<string, FindingKind> = {
  ContextBomb: FindingKind.ContextBomb,
  DeadWeight: FindingKind.DeadWeight,
  ReexportEntropy: FindingKind.ReexportEntropy,
  CouplingHotspot: FindingKind.CouplingHotspot,
  CodeDuplication: FindingKind.CodeDuplication,
  UnusedImport: FindingKind.UnusedImport,
};

const RUST_SEVERITY_MAP: Record<string, Severity> = {
  Critical: Severity.Critical,
  High: Severity.High,
  Medium: Severity.Medium,
  Low: Severity.Low,
};

function normalizeFindings(raw: unknown[]): Finding[] {
  return raw.map((f: any, i: number) => ({
    id: f.id || `rust-${String(i + 1).padStart(3, '0')}`,
    kind: RUST_KIND_MAP[f.kind] || f.kind,
    severity: RUST_SEVERITY_MAP[f.severity] || f.severity,
    confidence: f.confidence ?? 0.5,
    path: f.path || '',
    summary: f.summary || '',
    reasons: f.reasons || [],
    evidence: f.evidence || [],
    suggested_next_step: f.suggested_next_step || '',
    estimated_tokens: f.estimated_tokens ?? undefined,
  }));
}

interface RustBinaryOutput {
  findings: Finding[];
  errors: string[];
  filesAnalyzed?: number;
  totalLines?: number;
  totalEstimatedTokens?: number;
}

function spawnBinary(binaryPath: string, args: string[]): Promise<RustBinaryOutput> {
  return new Promise((resolve) => {
    const child = spawn(binaryPath, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';

    child.stdout!.on('data', (d: Buffer) => (stdout += d.toString()));
    child.stderr!.on('data', (d: Buffer) => (stderr += d.toString()));

    child.on('close', (code) => {
      if (code !== 0) {
        resolve({ findings: [], errors: [processExited('Rust binary', code, stderr.trim())] });
        return;
      }
      try {
        const parsed = JSON.parse(stdout);
        const rawFindings: unknown[] = parsed.findings || parsed || [];
        resolve({
          findings: normalizeFindings(rawFindings),
          errors: [],
          filesAnalyzed: parsed.files_analyzed,
          totalLines: parsed.total_lines,
          totalEstimatedTokens: parsed.total_estimated_tokens,
        });
      } catch {
        resolve({ findings: [], errors: [outputParseFailed('Rust binary', stdout)] });
      }
    });

    child.on('error', (err) => {
      resolve({ findings: [], errors: [spawnFailed('Rust binary', err.message)] });
    });
  });
}

function thresholdToArg(thresholds: Thresholds): string {
  if (thresholds.lineCountLimit <= 300) return 'strict';
  if (thresholds.lineCountLimit >= 500) return 'relaxed';
  return 'normal';
}

export const rustPlugin: LanguagePlugin = {
  name: 'rust',
  displayName: 'Rust',
  fileExtensions: ['.rs'],
  testFilePatterns: [],
  requiredManifests: ['Cargo.toml'],
  selfDiscovers: true,

  async analyzeLanguage(
    _files: SourceFile[],
    workspaceRoot: string,
    thresholds: Thresholds,
  ): Promise<AnalysisResult> {
    const binary = findBinary();
    if (!binary) {
      return { findings: [], skipped: 0, errors: [runtimeNotFound('Rust binary', 'Install repo-gc-rust alongside repo-gc, or build from source with `cargo build --release`.')] };
    }

    const args = [
      'scan',
      '--path', workspaceRoot,
      '--format', 'json',
      '--threshold', thresholdToArg(thresholds),
    ];

    return { ...(await spawnBinary(binary, args)), skipped: 0 };
  },
};
