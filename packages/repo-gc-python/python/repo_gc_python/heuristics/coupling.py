"""High fan-in/fan-out detection with pattern-aware output — mirrors Rust's coupling.rs."""

from ..cli import Threshold
from ..discovery import PythonFile
from ..graph import ImportGraph
from ..types import Finding, FindingKind, Severity


def analyze(
    pfile: PythonFile,
    graph: ImportGraph,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    fan_in = graph.fan_in.get(pfile.path, 0)
    fan_out = graph.fan_out.get(pfile.path, 0)
    in_limit = threshold.fan_in_limit
    out_limit = threshold.fan_out_limit

    if fan_in < in_limit and fan_out < out_limit:
        return None

    i = fan_out / (fan_in + fan_out) if (fan_in + fan_out) > 0 else 0.0
    fan_in_over = fan_in >= in_limit
    fan_out_over = fan_out >= out_limit

    if fan_in_over and not fan_out_over:
        # API/dispatch hub: high fan-in, low fan-out — intentionally stable
        severity = Severity.Medium
        confidence = 0.65
        summary = (
            f"Wide API surface — fan-in={fan_in}, fan-out={fan_out}, I={i:.2f}. "
            f"This looks like an intentional API/dispatch hub — "
            f"verify it's not accidental coupling."
        )
        suggested_next_step = (
            f"If this is an intentional API/dispatch hub, this is fine — "
            f"no action needed. Otherwise, split callers across focused "
            f"interfaces ({pfile.relative_path} → {fan_in} dependents)."
        )
        pattern_label = "api"
        i_interpretation = "very stable (API-like)" if i < 0.1 else "stable (API-like)"

    elif not fan_in_over and fan_out_over:
        # Over-orchestrator: high fan-out, low fan-in — depends on too many peers
        if fan_out >= out_limit * 3:
            severity = Severity.High
        else:
            severity = Severity.Medium
        confidence = 0.70
        summary = (
            f"High dependency fan-out ({fan_out}), low fan-in ({fan_in}), "
            f"I={i:.2f}. This module imports many peers — consider decomposing."
        )
        suggested_next_step = "Decompose into focused modules with fewer dependencies each."
        pattern_label = "orch"
        i_interpretation = "very unstable (consumer-like)" if i > 0.9 else "unstable (consumer-like)"

    else:
        # God module: both dimensions exceed thresholds
        if fan_in >= in_limit * 3 or fan_out >= out_limit * 2:
            severity = Severity.High
        else:
            severity = Severity.Medium
        confidence = 0.85
        summary = (
            f"Dependency concentration — fan-in={fan_in}, fan-out={fan_out}, "
            f"I={i:.2f}. High change-impact surface AND high dependency count."
        )
        suggested_next_step = (
            f"Extract an interface to decouple {pfile.relative_path} from its dependents."
        )
        pattern_label = "god"
        i_interpretation = "balanced"

    reasons: list[str] = []
    if fan_in_over:
        reasons.append(f"Fan-in: {fan_in} modules import this (limit: {in_limit})")
    if fan_out_over:
        reasons.append(f"Fan-out: imports {fan_out} modules (limit: {out_limit})")
    reasons.append(f"Instability I={i:.3f} — {i_interpretation}")

    return Finding(
        id=f"ch-{counter:03d}",
        kind=FindingKind.CouplingHotspot,
        severity=severity,
        confidence=confidence,
        path=str(pfile.relative_path),
        summary=summary,
        reasons=reasons,
        evidence=[
            f"fan_in: {fan_in}",
            f"fan_out: {fan_out}",
            f"instability: {i:.3f}",
            f"pattern: {pattern_label}",
        ],
        suggested_next_step=suggested_next_step,
        estimated_tokens=None,
    )
