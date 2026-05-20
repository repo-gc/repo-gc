import { Finding, FindingKind, Severity, type Thresholds } from 'repo-gc-shared';
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
  let summary: string;
  let suggestedNextStep: string;
  let patternLabel: string;
  let iInterpretation: string;

  if (fanInOver && !fanOutOver) {
    // API/dispatch hub: high fan-in, low fan-out — intentionally stable
    severity = Severity.Medium;
    confidence = 0.65;
    summary = `Wide API surface — fan-in=${fanIn}, fan-out=${fanOut}, I=${i.toFixed(2)}. This looks like an intentional API/dispatch hub — verify it's not accidental coupling.`;
    suggestedNextStep = `If this is an intentional API/dispatch hub, this is fine — no action needed. Otherwise, split callers across focused interfaces (${info.relativePath} → ${fanIn} dependents).`;
    patternLabel = 'api';
    iInterpretation = i < 0.1 ? 'very stable (API-like)' : 'stable (API-like)';
  } else if (!fanInOver && fanOutOver) {
    // Over-orchestrator: high fan-out, low fan-in — depends on too many peers
    severity = fanOut >= threshold.fanOutLimit * 3 ? Severity.High : Severity.Medium;
    confidence = 0.70;
    summary = `High dependency fan-out (${fanOut}), low fan-in (${fanIn}), I=${i.toFixed(2)}. This module imports many peers — consider decomposing.`;
    suggestedNextStep = 'Decompose into focused modules with fewer dependencies each.';
    patternLabel = 'orch';
    iInterpretation = i > 0.9 ? 'very unstable (consumer-like)' : 'unstable (consumer-like)';
  } else {
    // God module: both dimensions exceed thresholds
    severity = fanIn >= threshold.fanInLimit * 3 || fanOut >= threshold.fanOutLimit * 2
      ? Severity.High : Severity.Medium;
    confidence = 0.85;
    summary = `Dependency concentration — fan-in=${fanIn}, fan-out=${fanOut}, I=${i.toFixed(2)}. High change-impact surface AND high dependency count.`;
    suggestedNextStep = `Extract an interface to decouple ${info.relativePath} from its dependents.`;
    patternLabel = 'god';
    iInterpretation = 'balanced';
  }

  const reasons: string[] = [];
  if (fanInOver) {
    reasons.push(`Imported by ${fanIn} files (fan-in limit: ${threshold.fanInLimit})`);
  }
  if (fanOutOver) {
    reasons.push(`Imports ${fanOut} modules (fan-out limit: ${threshold.fanOutLimit})`);
  }
  reasons.push(`Instability I=${i.toFixed(3)} — ${iInterpretation}`);

  return [{
    id: `cp-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.CouplingHotspot,
    severity,
    confidence,
    path: info.relativePath,
    summary,
    reasons,
    evidence: [
      `fan_in: ${fanIn}`,
      `fan_out: ${fanOut}`,
      `instability: ${i.toFixed(3)}`,
      `pattern: ${patternLabel}`,
    ],
    suggested_next_step: suggestedNextStep,
  }];
}
