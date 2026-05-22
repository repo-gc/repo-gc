// Schema types — shared across all repo-gc analyzer packages.
// Mirrors Rust's types.rs for JSON/LLM-TSV schema compatibility.

export enum Severity {
  Critical = 'CRITICAL',
  High = 'HIGH',
  Medium = 'MEDIUM',
  Low = 'LOW',
}

export const SEVERITY_WEIGHT: Record<Severity, number> = {
  [Severity.Critical]: 4.0,
  [Severity.High]: 2.0,
  [Severity.Medium]: 1.0,
  [Severity.Low]: 0.5,
};

export const SEVERITY_LLM: Record<Severity, string> = {
  [Severity.Critical]: 'C',
  [Severity.High]: 'H',
  [Severity.Medium]: 'M',
  [Severity.Low]: 'L',
};

export enum FindingKind {
  ContextBomb = 'context-bomb',
  DeadWeight = 'dead-weight',
  ReexportEntropy = 'reexport-entropy',
  CouplingHotspot = 'coupling-hotspot',
  CodeDuplication = 'code-duplication',
  UnusedImport = 'unused-import',
  BranchDensity = 'branch-density',
  DeepNesting = 'deep-nesting',
  TypeComplexity = 'type-complexity',
  CommentRatio = 'comment-ratio',
  ImplicitControl = 'implicit-control',
  ErrorSwallow = 'error-swallow',
  DangerousPattern = 'dangerous-pattern',
  NamingEntropy = 'naming-entropy',
  StringlyTyped = 'stringly-typed',
  ImportDiversity = 'import-diversity',
}

export const FINDING_KIND_LLM: Record<FindingKind, string> = {
  [FindingKind.ContextBomb]: 'OVS',
  [FindingKind.DeadWeight]: 'DEAD',
  [FindingKind.ReexportEntropy]: 'EXP',
  [FindingKind.CouplingHotspot]: 'COUP',
  [FindingKind.CodeDuplication]: 'DUP',
  [FindingKind.UnusedImport]: 'ZOMB',
  [FindingKind.BranchDensity]: 'BRAN',
  [FindingKind.DeepNesting]: 'NEST',
  [FindingKind.TypeComplexity]: 'TYPE',
  [FindingKind.CommentRatio]: 'CMNT',
  [FindingKind.ImplicitControl]: 'HIDE',
  [FindingKind.ErrorSwallow]: 'SWAL',
  [FindingKind.DangerousPattern]: 'DANG',
  [FindingKind.NamingEntropy]: 'MIXD',
  [FindingKind.StringlyTyped]: 'STRY',
  [FindingKind.ImportDiversity]: 'GODF',
};

export interface Finding {
  id: string;
  kind: FindingKind;
  severity: Severity;
  confidence: number;
  path: string;
  summary: string;
  reasons: string[];
  evidence: string[];
  suggested_next_step: string;
  estimated_tokens?: number;
}

export interface GlobalScore {
  ai_friction_score: number;
  context_waste_score: number;
  structural_entropy_score: number;
  reasoning_complexity_score: number;
  context_waste_ratio: number;
  estimated_waste_pct: number;
}

export interface Report {
  findings: Finding[];
  global_score: GlobalScore;
  files_analyzed: number;
  files_skipped: number;
  total_lines: number;
  total_estimated_tokens: number;
  errors: string[];
  version: string;
}

// Re-export for heuristics that import from ../types
export type { Thresholds } from './threshold';

// Language-agnostic data interfaces for centralized heuristics.
// Plugins produce these from their language-specific parsers.

export interface FunctionBodyData {
  name: string;
  rawBody: string;
  /** Identifier-normalized + whitespace-stripped body for Type 2 clone detection. */
  normalizedBody: string;
}

export interface FileData {
  path: string;
  relativePath: string;
  lineCount: number;
  sizeBytes: number;
  modulePath: string;
  isEntryPoint: boolean;
  /** Package init file — __init__.py, mod.rs, lib.rs, index.ts */
  isPackageInit: boolean;
  functionCount: number;
  classCount: number;
  implBlockCount: number;
  /** Raw import specifiers for graph building (e.g. "./utils", "react") */
  imports: string[];
  /** Leaf names brought into scope by imports (for unused-import detection) */
  importedNames: string[];
  /** All identifiers referenced in the file body */
  allIdentifiers: Set<string>;
  /** Rust `mod` declarations — empty for non-Rust languages */
  moduleDeclarations: string[];
  /** Python `__all__` export list — null for non-Python languages */
  allExport: string[] | null;
  /** Re-export entries (barrel files) */
  exports: Array<{ sourcePath: string; itemCount: number; isWildcard: boolean }>;
  /** Function bodies for duplication detection */
  functionBodies: FunctionBodyData[];
  /** Branching density metrics */
  branchCount: number;
  /** Maximum nesting depth of control flow */
  maxNestingDepth: number;
  /** Maximum type expression depth */
  maxTypeDepth: number;
  /** Number of comment-only lines */
  commentLineCount: number;
  /** Number of decorator/annotation usages */
  decoratorCount: number;
  /** Number of empty catch blocks */
  emptyCatchCount: number;
  /** Number of dangerous pattern matches */
  dangerousPatternCount: number;
  /** Number of string comparison operations */
  stringComparisonCount: number;
}

/** Cross-file import graph — computed by plugins from FileData[].imports */
export interface GraphData {
  fanIn: Map<string, number>;
  fanOut: Map<string, number>;
}
