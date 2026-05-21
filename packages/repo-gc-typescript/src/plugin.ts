import type { Finding, LanguagePlugin, AnalysisResult, SourceFile, Thresholds } from 'repo-gc';
import {
  analyzeContextBombs,
  analyzeDeadWeight,
  analyzeCoupling,
  analyzeReexportEntropy,
  analyzeDuplication,
  analyzeUnusedImports,
} from 'repo-gc';
import { parseAllFiles } from './parser';
import { ImportGraph } from './graph';
import { toFileData } from './adapter';
import type { SourceFile as InternalSourceFile, FileInfo } from './types';

// Config patterns that are never orphaned (consumed by tooling, not app code)
const CONFIG_PATTERNS = [
  /\.config\.(ts|js|mjs|cjs)$/,
  /\.eslintrc\.(js|cjs|json)$/,
  /\.prettierrc\.(js|cjs|json)$/,
  /tailwind\.config\.(ts|js)$/,
  /vite\.config\.(ts|js)$/,
  /jest\.config\.(ts|js)$/,
];

// Names commonly referenced by compile-time transforms rather than user code
const COMPILER_NAMES = new Set(['React', 'Fragment', 'jsx', 'jsxs']);

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

    // Build file lookup for lineCount/sizeBytes
    const fileMap = new Map<string, InternalSourceFile>();
    for (const f of internalFiles) fileMap.set(f.path, f);

    // Convert to language-agnostic FileData
    const fileDataList = infos.map((info) => {
      const fd = toFileData(
        info,
        graph.fanIn.get(info.path) || 0,
        graph.fanOut.get(info.path) || 0,
      );
      // Fill in lineCount and sizeBytes from the original file
      const sf = fileMap.get(info.path);
      if (sf) {
        fd.lineCount = sf.lineCount;
        fd.sizeBytes = sf.sizeBytes;
      }
      return fd;
    });

    const graphData = { fanIn: graph.fanIn, fanOut: graph.fanOut };

    // Run shared heuristics
    const findings: Finding[] = [];
    const idCounter = { value: 1 };

    for (const fd of fileDataList) {
      try {
        const cb = analyzeContextBombs(fd, thresholds, idCounter);
        if (cb) findings.push(cb);
      } catch (e) {
        errors.push(`${fd.relativePath}: context-bombs: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const re = analyzeReexportEntropy(fd, thresholds, idCounter);
        if (re) findings.push(re);
      } catch (e) {
        errors.push(`${fd.relativePath}: reexport-entropy: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const cp = analyzeCoupling(fd, graphData, thresholds, idCounter);
        findings.push(...cp);
      } catch (e) {
        errors.push(`${fd.relativePath}: coupling: ${e instanceof Error ? e.message : String(e)}`);
      }

      try {
        const ui = analyzeUnusedImports(fd, idCounter, { compilerNames: COMPILER_NAMES });
        if (ui) findings.push(ui);
      } catch (e) {
        errors.push(`${fd.relativePath}: unused-imports: ${e instanceof Error ? e.message : String(e)}`);
      }
    }

    try {
      const dw = analyzeDeadWeight(fileDataList, graphData, idCounter, {
        configPatterns: CONFIG_PATTERNS,
      });
      findings.push(...dw);
    } catch (e) {
      errors.push(`dead-weight: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const dup = analyzeDuplication(fileDataList, idCounter, {
        testNamePatterns: ['for_test', 'forTest'],
      });
      findings.push(...dup);
    } catch (e) {
      errors.push(`duplication: ${e instanceof Error ? e.message : String(e)}`);
    }

    return { findings, skipped: skippedCount, errors };
  },
};
