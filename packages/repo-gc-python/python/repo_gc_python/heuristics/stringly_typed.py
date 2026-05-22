"""Stringly-typed heuristic.

Detects magic string comparisons — string literals used in comparisons and
match patterns where enums or constants should be used instead.
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
    limit = threshold.string_comparison_limit
    if info.string_comparison_count <= limit:
        return None

    if info.string_comparison_count >= limit * 3:
        severity = Severity.High
    elif info.string_comparison_count >= limit * 2:
        severity = Severity.Medium
    else:
        severity = Severity.Low

    return Finding(
        id=f"st-{counter:03d}",
        kind=FindingKind.StringlyTyped,
        severity=severity,
        confidence=0.65,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"string_comparison_count: {info.string_comparison_count}",
            f"limit: {limit}",
            f"preview: {pfile.relative_path}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
