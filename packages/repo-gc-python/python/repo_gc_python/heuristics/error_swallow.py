"""Empty error handler detection -- #1 LLM-generated anti-pattern (CATCHALL paper)."""

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
    limit = threshold.empty_catch_limit
    if info.empty_catch_count <= limit:
        return None

    if info.empty_catch_count >= 5:
        severity = Severity.High
    elif info.empty_catch_count >= 3:
        severity = Severity.Medium
    else:
        severity = Severity.Low

    return Finding(
        id=f"es-{counter:03d}",
        kind=FindingKind.ErrorSwallow,
        severity=severity,
        confidence=0.75,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"empty_catch_count: {info.empty_catch_count}",
            f"limit: {limit}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
