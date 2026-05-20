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
}

export const FINDING_KIND_LLM: Record<FindingKind, string> = {
  [FindingKind.ContextBomb]: 'OVS',
  [FindingKind.DeadWeight]: 'DEAD',
  [FindingKind.ReexportEntropy]: 'EXP',
  [FindingKind.CouplingHotspot]: 'COUP',
  [FindingKind.CodeDuplication]: 'DUP',
  [FindingKind.UnusedImport]: 'ZOMB',
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
