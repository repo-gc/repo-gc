"""Deep nesting heuristic — measures maximum control-flow nesting depth per file.

Deeply nested code increases cognitive load for LLMs; they skip condition
bodies and miss deeply nested blocks (RE2-Bench: significant perf drop at
depths > 4).
"""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    limit = threshold.nesting_depth_limit
    depth = info.max_nesting_depth
    if depth <= limit:
        return None

    if depth >= limit + 4:
        severity = Severity.Critical
    elif depth >= limit + 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    return Finding(
        id=f"dn-{counter:03d}",
        kind=FindingKind.DeepNesting,
        severity=severity,
        confidence=0.85,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"max_depth: {depth}",
            f"limit: {limit}",
            "deepest_at: unknown",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
