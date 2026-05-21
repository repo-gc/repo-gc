import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc';
import { FileInfo, ImportGraph } from '../types';

export function analyze(
  info: FileInfo,
  graph: ImportGraph,
  threshold: Thresholds,
  idCounter: { value: number },
): Finding[] {
  const fanIn = graph.fanIn.get(info.path) || 0;
  const fanOut = graph.fanOut.get(info.path) || 0;

  if (fanIn < threshold.fanInLimit && fanOut < threshold.fanOutLimit) {
    return [];
  }

  const i = fanIn + fanOut > 0 ? fanOut / (fanIn + fanOut) : 0;
  const fanInOver = fanIn >= threshold.fanInLimit;
  const fanOutOver = fanOut >= threshold.fanOutLimit;

  let severity: Severity;
  let confidence: number;
  let patternLabel: string;

  if (fanInOver && !fanOutOver) {
    severity = Severity.Medium;
    confidence = 0.65;
    patternLabel = 'api';
  } else if (!fanInOver && fanOutOver) {
    severity = fanOut >= threshold.fanOutLimit * 3 ? Severity.High : Severity.Medium;
    confidence = 0.70;
    patternLabel = 'orch';
  } else {
    severity = fanIn >= threshold.fanInLimit * 3 || fanOut >= threshold.fanOutLimit * 2
      ? Severity.High : Severity.Medium;
    confidence = 0.85;
    patternLabel = 'god';
  }

  return [{
    id: `cp-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.CouplingHotspot,
    severity,
    confidence,
    path: info.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `fan_in: ${fanIn}`,
      `fan_out: ${fanOut}`,
      `instability: ${i.toFixed(3)}`,
      `pattern: ${patternLabel}`,
    ],
    suggested_next_step: '',
  }];
}
