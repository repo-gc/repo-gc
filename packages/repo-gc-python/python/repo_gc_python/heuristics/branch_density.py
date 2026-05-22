"""Branch density heuristic.

Measures cyclomatic complexity per function. High branch density degrades
LLM reasoning performance (RE2-Bench: 51.5% perf drop from low to high
complexity).

Logic: branch_count / max(function_count, 1) > limit -> finding.
Severity: ratio >= limit*3 -> Critical, >= limit*2 -> High, >= limit -> Medium.
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
    limit = threshold.branch_density_limit
    fn_count = max(info.function_count, 1)
    ratio = info.branch_count / fn_count

    if ratio <= limit:
        return None

    if ratio >= limit * 3:
        severity = Severity.Critical
    elif ratio >= limit * 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    return Finding(
        id=f"bd-{counter:03d}",
        kind=FindingKind.BranchDensity,
        severity=severity,
        confidence=0.85,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"branch_count: {info.branch_count}",
            f"function_count: {fn_count}",
            f"avg_branches_per_fn: {ratio:.2f}",
            f"limit: {limit}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
