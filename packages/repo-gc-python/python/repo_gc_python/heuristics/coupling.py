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
        severity = Severity.Medium
        confidence = 0.65
        pattern_label = "api"
    elif not fan_in_over and fan_out_over:
        if fan_out >= out_limit * 3:
            severity = Severity.High
        else:
            severity = Severity.Medium
        confidence = 0.70
        pattern_label = "orch"
    else:
        if fan_in >= in_limit * 3 or fan_out >= out_limit * 2:
            severity = Severity.High
        else:
            severity = Severity.Medium
        confidence = 0.85
        pattern_label = "god"

    return Finding(
        id=f"ch-{counter:03d}",
        kind=FindingKind.CouplingHotspot,
        severity=severity,
        confidence=confidence,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"fan_in: {fan_in}",
            f"fan_out: {fan_out}",
            f"instability: {i:.3f}",
            f"pattern: {pattern_label}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
