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

const THRESHOLD_MAP: Record<ThresholdLevel, Thresholds> = {
  strict: {
    lineCountLimit: 300, fanInLimit: 5, fanOutLimit: 10, reexportLimit: 5,
    branchDensityLimit: 8, nestingDepthLimit: 4, typeDepthLimit: 3,
    commentRatioMin: 0.03, commentRatioMax: 0.20, decoratorDensityLimit: 0.33,
    emptyCatchLimit: 1, dangerousPatternLimit: 3,
    stringComparisonLimit: 5, importDomainLimit: 8,
  },
  normal: {
    lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10,
    branchDensityLimit: 12, nestingDepthLimit: 6, typeDepthLimit: 4,
    commentRatioMin: 0.03, commentRatioMax: 0.30, decoratorDensityLimit: 0.50,
    emptyCatchLimit: 2, dangerousPatternLimit: 5,
    stringComparisonLimit: 10, importDomainLimit: 10,
  },
  relaxed: {
    lineCountLimit: 1000, fanInLimit: 20, fanOutLimit: 25, reexportLimit: 20,
    branchDensityLimit: 18, nestingDepthLimit: 8, typeDepthLimit: 5,
    commentRatioMin: 0.01, commentRatioMax: 0.50, decoratorDensityLimit: 0.75,
    emptyCatchLimit: 3, dangerousPatternLimit: 10,
    stringComparisonLimit: 15, importDomainLimit: 14,
  },
};

export function getThresholds(level: ThresholdLevel): Thresholds {
  return THRESHOLD_MAP[level];
}
