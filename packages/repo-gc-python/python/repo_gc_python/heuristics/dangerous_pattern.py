"""Dangerous pattern heuristic.

Detects risky code patterns (eval, exec, compile, __import__) that force
LLMs to consider a broader state space.
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
    limit = threshold.dangerous_pattern_limit
    if info.dangerous_pattern_count <= limit:
        return None

    if info.dangerous_pattern_count >= limit * 3:
        severity = Severity.Critical
    elif info.dangerous_pattern_count >= limit * 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    return Finding(
        id=f"dp-{counter:03d}",
        kind=FindingKind.DangerousPattern,
        severity=severity,
        confidence=0.70,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"dangerous_pattern_count: {info.dangerous_pattern_count}",
            f"limit: {limit}",
            f"preview: {pfile.relative_path}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
