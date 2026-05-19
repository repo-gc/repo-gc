export type ThresholdLevel = 'strict' | 'normal' | 'relaxed';

export interface Thresholds {
  lineCountLimit: number;
  fanInLimit: number;
  fanOutLimit: number;
  reexportLimit: number;
}

const THRESHOLD_MAP: Record<ThresholdLevel, Thresholds> = {
  strict: { lineCountLimit: 300, fanInLimit: 5, fanOutLimit: 10, reexportLimit: 5 },
  normal: { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 },
  relaxed: { lineCountLimit: 1000, fanInLimit: 20, fanOutLimit: 25, reexportLimit: 20 },
};

export function getThresholds(level: ThresholdLevel): Thresholds {
  return THRESHOLD_MAP[level];
}
