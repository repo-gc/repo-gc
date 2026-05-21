import type { Finding, FileData, GraphData } from '../types';
import type { Thresholds } from '../threshold';
import { analyzeContextBombs } from './context-bombs';
import { analyzeReexportEntropy } from './reexport-entropy';
import { analyzeCoupling } from './coupling';
import { analyzeUnusedImports } from './unused-imports';
import { analyzeDeadWeight } from './dead-weight';
import { analyzeDuplication } from './duplication';

export interface PluginHeuristicOptions {
  compilerNames?: Set<string>;
  configPatterns?: RegExp[];
  testNamePatterns?: string[];
}

export interface HeuristicsResult {
  findings: Finding[];
  errors: string[];
}

export function runHeuristics(
  fileDataList: FileData[],
  graphData: GraphData,
  thresholds: Thresholds,
  options: PluginHeuristicOptions = {},
): HeuristicsResult {
  const findings: Finding[] = [];
  const errors: string[] = [];
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
      const ui = analyzeUnusedImports(fd, idCounter, { compilerNames: options.compilerNames });
      if (ui) findings.push(ui);
    } catch (e) {
      errors.push(`${fd.relativePath}: unused-imports: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  try {
    const dw = analyzeDeadWeight(fileDataList, graphData, idCounter, {
      configPatterns: options.configPatterns,
    });
    findings.push(...dw);
  } catch (e) {
    errors.push(`dead-weight: ${e instanceof Error ? e.message : String(e)}`);
  }

  try {
    const dup = analyzeDuplication(fileDataList, idCounter, {
      testNamePatterns: options.testNamePatterns,
    });
    findings.push(...dup);
  } catch (e) {
    errors.push(`duplication: ${e instanceof Error ? e.message : String(e)}`);
  }

  return { findings, errors };
}
