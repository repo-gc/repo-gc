import type { Finding, LanguagePlugin, AnalysisResult, SourceFile, Thresholds } from 'repo-gc';
import { FindingKind, Severity } from 'repo-gc';
import { parseAllFiles } from './parser';
import { ImportGraph } from './graph';
import {
  analyzeContextBombs,
  analyzeDeadWeight,
  analyzeCoupling,
  analyzeReexportEntropy,
  analyzeDuplication,
  analyzeUnusedImports,
} from './heuristics';
import type { SourceFile as InternalSourceFile, FileInfo } from './types';

function toInternalFile(f: SourceFile): InternalSourceFile {
  return {
    path: f.path,
    relativePath: f.relativePath,
    packageName: f.metadata['npmPackage'] || '',
    sizeBytes: f.sizeBytes,
    lineCount: f.lineCount,
  };
}

export const typescriptPlugin: LanguagePlugin = {
  name: 'typescript',
  displayName: 'TypeScript/JavaScript',
  fileExtensions: ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs'],
  testFilePatterns: [/\.test\./, /\.spec\./, /__tests__\//, /^tests\//],
  requiredManifests: ['package.json'],

  analyzeLanguage(
    files: SourceFile[],
    workspaceRoot: string,
    thresholds: Thresholds,
  ): AnalysisResult {
    const internalFiles = files.map(toInternalFile);
    const errors: string[] = [];

    // Parse files
    const { parsed, skippedCount } = parseAllFiles(internalFiles, workspaceRoot);
    const parseResults = parsed.filter((p) => !p.skipped);
    const infos: FileInfo[] = parseResults.map((p) => p.info);

    // Build import graph
    const graph = ImportGraph.build(infos, workspaceRoot);

    // Build file lookup
    const fileMap = new Map<string, InternalSourceFile>();
    for (const f of internalFiles) fileMap.set(f.path, f);

    // Run heuristics
    const findings: Finding[] = [];
    const idCounter = { value: 1 };

    for (const info of infos) {
      const file = fileMap.get(info.path);
      if (!file) continue;

      try {
        const cb = analyzeContextBombs(file, info, thresholds, idCounter);
        if (cb) findings.push(cb);
      } catch (e) {
        errors.push(`${info.relativePath}: context-bombs: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const rx = analyzeReexportEntropy(info, thresholds, idCounter);
        if (rx) findings.push(rx);
      } catch (e) {
        errors.push(`${info.relativePath}: reexport-entropy: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const cp = analyzeCoupling(info, graph, thresholds, idCounter);
        findings.push(...cp);
      } catch (e) {
        errors.push(`${info.relativePath}: coupling: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const ui = analyzeUnusedImports(info, thresholds, idCounter);
        if (ui) findings.push(ui);
      } catch (e) {
        errors.push(`${info.relativePath}: unused-imports: ${e instanceof Error ? e.message : String(e)}`);
      }
    }

    try {
      const dw = analyzeDeadWeight(internalFiles, infos, graph, idCounter);
      findings.push(...dw);
    } catch (e) {
      errors.push(`dead-weight: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const dup = analyzeDuplication(infos, idCounter);
      findings.push(...dup);
    } catch (e) {
      errors.push(`duplication: ${e instanceof Error ? e.message : String(e)}`);
    }

    return { findings, skipped: skippedCount, errors };
  },
};
