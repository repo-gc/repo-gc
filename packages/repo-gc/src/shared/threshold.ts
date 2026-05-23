import canonical from '../../../../test-fixtures/finding-kinds.json' with { type: 'json' };

export type ThresholdLevel = 'strict' | 'normal' | 'relaxed';

export interface Thresholds {
  lineCountLimit: number;
  fanInLimit: number;
  fanOutLimit: number;
  reexportLimit: number;
  branchDensityLimit: number;
  nestingDepthLimit: number;
  typeDepthLimit: number;
  commentRatioMin: number;
  commentRatioMax: number;
  decoratorDensityLimit: number;
  emptyCatchLimit: number;
  dangerousPatternLimit: number;
  stringComparisonLimit: number;
  importDomainLimit: number;
}

function snakeToCamel(s: string): string {
  return s.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
}

function buildThresholds(raw: Record<string, number>): Thresholds {
  const result: Record<string, number> = {};
  for (const key of Object.keys(raw)) {
    result[snakeToCamel(key)] = raw[key];
  }
  return result as unknown as Thresholds;
}

const THRESHOLD_MAP: Record<ThresholdLevel, Thresholds> = {
  strict: buildThresholds(canonical.thresholds.strict),
  normal: buildThresholds(canonical.thresholds.normal),
  relaxed: buildThresholds(canonical.thresholds.relaxed),
};

export function getThresholds(level: ThresholdLevel): Thresholds {
  return THRESHOLD_MAP[level];
}
