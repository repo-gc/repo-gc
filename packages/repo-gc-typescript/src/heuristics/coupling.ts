import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc-shared';
import { FileInfo, ImportGraph } from '../types';

export function analyze(
  info: FileInfo,
  graph: ImportGraph,
  threshold: Thresholds,
  idCounter: { value: number },
): Finding[] {
  const findings: Finding[] = [];

  const fanIn = graph.fanIn.get(info.path) || 0;
  const fanOut = graph.fanOut.get(info.path) || 0;

  const reasons: string[] = [];
  const evidence: string[] = [];
  let maxSeverity = Severity.Low;
  let maxConfidence = 0.0;

  if (fanIn > threshold.fanInLimit) {
    const ratio = fanIn / threshold.fanInLimit;
    if (ratio >= 3) {
      maxSeverity = Severity.High;
      maxConfidence = 0.85;
    } else {
      maxSeverity = Severity.Medium;
      maxConfidence = 0.85;
    }
    reasons.push(`Imported by ${fanIn} files (fan-in limit: ${threshold.fanInLimit})`);
    evidence.push(`fan-in=${fanIn}`);
  }

  if (fanOut > threshold.fanOutLimit) {
    const ratio = fanOut / threshold.fanOutLimit;
    if (ratio >= 2) {
      maxSeverity = Severity.High;
      maxConfidence = Math.max(maxConfidence, 0.85);
    } else {
      maxSeverity = maxSeverity === Severity.High ? Severity.High : Severity.Medium;
      maxConfidence = Math.max(maxConfidence, 0.85);
    }
    reasons.push(`Imports ${fanOut} modules (fan-out limit: ${threshold.fanOutLimit})`);
    evidence.push(`fan-out=${fanOut}`);
  }

  if (reasons.length === 0) return findings;

  findings.push({
    id: `cp-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.CouplingHotspot,
    severity: maxSeverity,
    confidence: maxConfidence,
    path: info.relativePath,
    summary: `High coupling: fan-in=${fanIn}, fan-out=${fanOut}`,
    reasons,
    evidence,
    suggested_next_step:
      'Extract an interface to decouple, or split the module into focused units',
  });

  return findings;
}
