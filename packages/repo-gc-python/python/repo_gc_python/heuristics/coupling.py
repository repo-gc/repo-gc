"""High fan-in/fan-out detection — mirrors Rust's coupling.rs."""

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

    if fan_in >= in_limit * 3 or fan_out >= out_limit * 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    reasons: list[str] = []
    if fan_in >= in_limit:
        reasons.append(f"Fan-in: {fan_in} modules import this (limit: {in_limit})")
    if fan_out >= out_limit:
        reasons.append(f"Fan-out: imports {fan_out} modules (limit: {out_limit})")

    return Finding(
        id=f"ch-{counter:03d}",
        kind=FindingKind.CouplingHotspot,
        severity=severity,
        confidence=0.85,
        path=str(pfile.relative_path),
        summary=(
            f"Dependency concentration — fan-in={fan_in}, fan-out={fan_out} "
            f"increases LLM reasoning overhead"
        ),
        reasons=reasons,
        evidence=[
            f"fan_in: {fan_in}",
            f"fan_out: {fan_out}",
        ],
        suggested_next_step=(
            f"Extract an interface to decouple {pfile.relative_path} from its dependents"
        ),
        estimated_tokens=None,
    )
