export { scan } from './runner';
export type { ScanOptions } from './runner';
export type { LanguagePlugin, AnalysisResult, SourceFile } from './plugin';
export { discoverWorkspace, enumerateFiles, detectLanguages } from './discovery';
// Shared types — re-exported so plugin packages only need to depend on repo-gc
export type {
  Finding,
  GlobalScore,
  Report,
  FunctionBodyData,
  FileData,
  GraphData,
} from './shared/types';
export { Severity, SEVERITY_WEIGHT, SEVERITY_LLM, FindingKind, FINDING_KIND_LLM } from './shared/types';
export type { ThresholdLevel, Thresholds } from './shared/threshold';
export { getThresholds } from './shared/threshold';
export { estimateTokens } from './shared/token-estimate';
export { runtimeNotFound, pluginDirNotFound, processExited, outputParseFailed, spawnFailed } from './shared/messages';

// Shared heuristics
export {
  analyzeContextBombs,
  analyzeDeadWeight,
  analyzeCoupling,
  analyzeReexportEntropy,
  analyzeDuplication,
  analyzeUnusedImports,
} from './shared/heuristics';
export type {
  DeadWeightOptions,
  DuplicationOptions,
  UnusedImportsOptions,
} from './shared/heuristics';
