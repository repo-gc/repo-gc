export {
  Severity,
  SEVERITY_WEIGHT,
  SEVERITY_LLM,
  FindingKind,
  FINDING_KIND_LLM,
} from './types';
export type {
  Finding,
  GlobalScore,
  Report,
  FunctionBodyData,
  FileData,
  GraphData,
} from './types';

export { type ThresholdLevel, type Thresholds, getThresholds } from './threshold';

export { estimateTokens } from './token-estimate';

export { computeGlobalScore } from './scoring';

export { renderReport } from './render';
export { renderTerminal, renderJson, renderMarkdown, renderLlm } from './reporters';

export type { SourceFile, AnalysisResult, LanguagePlugin } from './plugin';

// Centralised user-facing messages for plugins
export {
  runtimeNotFound,
  pluginDirNotFound,
  processExited,
  outputParseFailed,
  spawnFailed,
} from './messages';

// Shared heuristic functions (language-agnostic)
export {
  analyzeContextBombs,
  analyzeDeadWeight,
  analyzeCoupling,
  analyzeReexportEntropy,
  analyzeDuplication,
  analyzeUnusedImports,
  analyzeErrorSwallow,
  analyzeNamingEntropy,
} from './heuristics';
export type {
  DeadWeightOptions,
  DuplicationOptions,
  UnusedImportsOptions,
} from './heuristics';
