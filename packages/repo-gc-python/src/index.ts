import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve, join, delimiter } from 'node:path';
import { existsSync } from 'node:fs';
import type { LanguagePlugin, AnalysisResult, SourceFile } from 'repo-gc-shared';
import { FindingKind, Severity, pluginDirNotFound, runtimeNotFound, processExited, outputParseFailed, spawnFailed } from 'repo-gc-shared';
import type { Finding, Thresholds } from 'repo-gc-shared';

const __dirname = dirname(fileURLToPath(import.meta.url));

function findPythonDir(): string | null {
  // __dirname is packages/repo-gc-python/dist/ (built) or src/ (dev)
  const candidates = [
    resolve(__dirname, '../python'),
    resolve(__dirname, '../../repo-gc-python/python'),
  ];
  for (const d of candidates) {
    const init = join(d, 'repo_gc_python', '__init__.py');
    if (existsSync(init)) return d;
  }
  return null;
}

function findPythonBin(): string | null {
  for (const name of ['python3', 'python']) {
    const pathDirs = (process.env.PATH || '').split(delimiter);
    for (const dir of pathDirs) {
      const p = join(dir, name);
      if (p !== name && existsSync(p)) return name;
    }
  }
  return null;
}

const PYTHON_KIND_MAP: Record<string, FindingKind> = {
  ContextBomb: FindingKind.ContextBomb,
  DeadWeight: FindingKind.DeadWeight,
  ReexportEntropy: FindingKind.ReexportEntropy,
  CouplingHotspot: FindingKind.CouplingHotspot,
  CodeDuplication: FindingKind.CodeDuplication,
  UnusedImport: FindingKind.UnusedImport,
};

const PYTHON_SEVERITY_MAP: Record<string, Severity> = {
  Critical: Severity.Critical,
  High: Severity.High,
  Medium: Severity.Medium,
  Low: Severity.Low,
};

function normalizeFindings(raw: unknown[]): Finding[] {
  return raw.map((f: any, i: number) => ({
    id: f.id || `py-${String(i + 1).padStart(3, '0')}`,
    kind: PYTHON_KIND_MAP[f.kind] || f.kind,
    severity: PYTHON_SEVERITY_MAP[f.severity] || f.severity,
    confidence: f.confidence ?? 0.5,
    path: f.path || '',
    summary: f.summary || '',
    reasons: f.reasons || [],
    evidence: f.evidence || [],
    suggested_next_step: f.suggested_next_step || '',
    estimated_tokens: f.estimated_tokens ?? undefined,
  }));
}

interface PythonOutput {
  findings: Finding[];
  filesAnalyzed?: number;
  totalLines?: number;
  totalEstimatedTokens?: number;
  errors?: string[];
}

function spawnPython(
  pythonBin: string,
  pyDir: string,
  args: string[],
): Promise<PythonOutput> {
  return new Promise((resolve) => {
    const child = spawn(pythonBin, ['-m', 'repo_gc_python', ...args], {
      cwd: pyDir,
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, PYTHONPATH: pyDir },
    });

    let stdout = '';
    let stderr = '';

    const timer = setTimeout(() => {
      child.kill('SIGTERM');
    }, 60_000);

    const onSigint = () => { child.kill('SIGINT'); process.exit(130); };
    const onSigterm = () => { child.kill('SIGTERM'); process.exit(143); };
    process.on('SIGINT', onSigint);
    process.on('SIGTERM', onSigterm);

    child.stdout!.on('data', (d: Buffer) => (stdout += d.toString()));
    child.stderr!.on('data', (d: Buffer) => (stderr += d.toString()));

    child.on('close', (code) => {
      clearTimeout(timer);
      process.off('SIGINT', onSigint);
      process.off('SIGTERM', onSigterm);

      if (code !== 0) {
        resolve({
          findings: [],
          filesAnalyzed: 0,
          totalLines: 0,
          totalEstimatedTokens: 0,
          errors: [processExited('Python', code, stderr.trim())],
        });
        return;
      }
      try {
        const parsed = JSON.parse(stdout);
        const rawFindings: unknown[] = parsed.findings || [];
        resolve({
          findings: normalizeFindings(rawFindings),
          filesAnalyzed: parsed.files_analyzed ?? 0,
          totalLines: parsed.total_lines ?? 0,
          totalEstimatedTokens: parsed.total_estimated_tokens ?? 0,
        });
      } catch {
        resolve({
          findings: [],
          filesAnalyzed: 0,
          totalLines: 0,
          totalEstimatedTokens: 0,
          errors: [outputParseFailed('Python', stderr.trim() || stdout)],
        });
      }
    });

    child.on('error', (err) => {
      clearTimeout(timer);
      process.off('SIGINT', onSigint);
      process.off('SIGTERM', onSigterm);
      resolve({
        findings: [],
        filesAnalyzed: 0,
        totalLines: 0,
        totalEstimatedTokens: 0,
        errors: [spawnFailed('Python', err.message)],
      });
    });
  });
}

function thresholdToArg(thresholds: Thresholds): string {
  if (thresholds.lineCountLimit <= 300) return 'strict';
  if (thresholds.lineCountLimit >= 1000) return 'relaxed';
  return 'normal';
}

export const pythonPlugin: LanguagePlugin = {
  name: 'python',
  displayName: 'Python',
  fileExtensions: ['.py'],
  testFilePatterns: [/\.test\./, /test_/, /_test/, /conftest\.py/],
  requiredManifests: ['pyproject.toml', 'setup.py', 'setup.cfg'],
  selfDiscovers: true,

  async analyzeLanguage(
    _files: SourceFile[],
    workspaceRoot: string,
    thresholds: Thresholds,
  ): Promise<AnalysisResult> {
    const pyDir = findPythonDir();
    if (!pyDir) {
      return {
        findings: [],
        skipped: 0,
        errors: [pluginDirNotFound('Python', 'Install repo-gc-python alongside repo-gc.')],
      };
    }

    const pythonBin = findPythonBin();
    if (!pythonBin) {
      return {
        findings: [],
        skipped: 0,
        errors: [runtimeNotFound('Python interpreter', 'Install Python >=3.10 and ensure it is on PATH.')],
      };
    }

    const args = [
      'scan',
      '--path', workspaceRoot,
      '--format', 'json',
      '--threshold', thresholdToArg(thresholds),
    ];

    const output = await spawnPython(pythonBin, pyDir, args);
    return {
      ...output,
      skipped: 0,
      errors: output.errors ?? [],
    };
  },
};
