import { Finding, FindingKind, Severity, type Thresholds, type FileData, type GraphData } from '../types';

export interface CouplingResult {
  finding: Finding;
}

export function analyzeCoupling(
  data: FileData,
  graph: GraphData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding[] {
  const fanIn = graph.fanIn.get(data.path) || 0;
  const fanOut = graph.fanOut.get(data.path) || 0;

  if (fanIn < thresholds.fanInLimit && fanOut < thresholds.fanOutLimit) {
    return [];
  }

  const i = fanIn + fanOut > 0 ? fanOut / (fanIn + fanOut) : 0;
  const fanInOver = fanIn >= thresholds.fanInLimit;
  const fanOutOver = fanOut >= thresholds.fanOutLimit;

  let severity: Severity;
  let confidence: number;
  let patternLabel: string;

  if (fanInOver && !fanOutOver) {
    severity = Severity.Medium;
    confidence = 0.65;
    patternLabel = 'api';
  } else if (!fanInOver && fanOutOver) {
    severity = fanOut >= thresholds.fanOutLimit * 3 ? Severity.High : Severity.Medium;
    confidence = 0.7;
    patternLabel = 'orch';
  } else {
    severity =
      fanIn >= thresholds.fanInLimit * 3 || fanOut >= thresholds.fanOutLimit * 2
        ? Severity.High
        : Severity.Medium;
    confidence = 0.85;
    patternLabel = 'god';
  }

  return [
    {
      id: `ch-${String(idCounter.value++).padStart(3, '0')}`,
      kind: FindingKind.CouplingHotspot,
      severity,
      confidence,
      path: data.relativePath,
      summary: '',
      reasons: [],
      evidence: [
        `fan_in: ${fanIn}`,
        `fan_out: ${fanOut}`,
        `instability: ${i.toFixed(3)}`,
        `pattern: ${patternLabel}`,
      ],
      suggested_next_step: '',
    },
  ];
}
