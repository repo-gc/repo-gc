"""Type complexity heuristic — detects deeply nested generic/union types.

High FP risk — only fires at extreme depths (>5).
Counts maximum nesting depth of type expressions (e.g.
``Optional[Dict[str, List[int]]]`` = depth 3).
"""

from __future__ import annotations

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
    depth = info.max_type_depth
    limit = threshold.type_depth_limit

    if depth <= limit:
        return None

    if depth >= limit + 3:
        severity = Severity.Critical
    elif depth >= limit + 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    return Finding(
        id=f"tc-{counter:03d}",
        kind=FindingKind.TypeComplexity,
        severity=severity,
        confidence=0.50,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"max_type_depth: {depth}",
            f"limit: {limit}",
            "location: unknown",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
