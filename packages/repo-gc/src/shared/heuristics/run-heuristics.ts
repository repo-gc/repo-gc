import type { Finding, FileData, GraphData } from '../types';
import type { Thresholds } from '../threshold';
import { analyzeImportDiversity } from './import-diversity';
import { analyzeContextBombs } from './context-bombs';
import { analyzeReexportEntropy } from './reexport-entropy';
import { analyzeCoupling } from './coupling';
import { analyzeUnusedImports } from './unused-imports';
import { analyzeBranchDensity } from './branch-density';
import { analyzeErrorSwallow } from './error-swallow';
import { analyzeDeepNesting } from './deep-nesting';
import { analyzeDangerousPattern } from './dangerous-pattern';
import { analyzeDeadWeight } from './dead-weight';
import { analyzeDuplication } from './duplication';
import { analyzeCommentRatio } from './comment-ratio';
import { analyzeStringlyTyped } from './stringly-typed';
import { analyzeNamingEntropy } from './naming-entropy';
import { analyzeTypeComplexity } from './type-complexity';
import { analyzeImplicitControl } from './implicit-control';

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

    try {
      const es = analyzeErrorSwallow(fd, thresholds, idCounter);
      if (es) findings.push(es);
    } catch (e) {
      errors.push(`${fd.relativePath}: error-swallow: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const bd = analyzeBranchDensity(fd, thresholds, idCounter);
      if (bd) findings.push(bd);
    } catch (e) {
      errors.push(`${fd.relativePath}: branch-density: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const dn = analyzeDeepNesting(fd, thresholds, idCounter);
      if (dn) findings.push(dn);
    } catch (e) {
      errors.push(`${fd.relativePath}: deep-nesting: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const idv = analyzeImportDiversity(fd, thresholds, idCounter);
      if (idv) findings.push(idv);
    } catch (e) {
      errors.push(`${fd.relativePath}: import-diversity: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const dp = analyzeDangerousPattern(fd, thresholds, idCounter);
      if (dp) findings.push(dp);
    } catch (e) {
      errors.push(`${fd.relativePath}: dangerous-pattern: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const cr = analyzeCommentRatio(fd, thresholds, idCounter);
      if (cr) findings.push(cr);
    } catch (e) {
      errors.push(`${fd.relativePath}: comment-ratio: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const st = analyzeStringlyTyped(fd, thresholds, idCounter);
      if (st) findings.push(st);
    } catch (e) {
      errors.push(`${fd.relativePath}: stringly-typed: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const ne = analyzeNamingEntropy(fd, thresholds, idCounter);
      if (ne) findings.push(ne);
    } catch (e) {
      errors.push(`${fd.relativePath}: naming-entropy: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const tc = analyzeTypeComplexity(fd, thresholds, idCounter);
      if (tc) findings.push(tc);
    } catch (e) {
      errors.push(`${fd.relativePath}: type-complexity: ${e instanceof Error ? e.message : String(e)}`);
    }

    try {
      const ic = analyzeImplicitControl(fd, thresholds, idCounter);
      if (ic) findings.push(ic);
    } catch (e) {
      errors.push(`${fd.relativePath}: implicit-control: ${e instanceof Error ? e.message : String(e)}`);
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
