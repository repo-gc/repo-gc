"""Implicit control flow heuristic — detects high decorator/middleware density.

Decorators create hidden execution paths that LLMs may miss.
Framework-aware: dataclass/pydantic-heavy files get 1.5x threshold multiplier.
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
    decorator_count = info.decorator_count
    fn_count = max(info.function_count, 1)
    ratio = decorator_count / fn_count

    # Framework-aware: dataclass/pydantic-heavy files get 1.5x threshold
    effective_limit = _effective_limit(threshold, info)

    if ratio <= effective_limit:
        return None

    if ratio >= effective_limit * 3:
        severity = Severity.High
    elif ratio > effective_limit * 2:
        severity = Severity.Medium
    else:
        severity = Severity.Low

    return Finding(
        id=f"ic-{counter:03d}",
        kind=FindingKind.ImplicitControl,
        severity=severity,
        confidence=0.40,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"decorator_count: {decorator_count}",
            f"function_count: {fn_count}",
            f"ratio: {ratio:.3f}",
            f"limit: {effective_limit:.3f}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )


def _effective_limit(threshold: Threshold, info: FileInfo) -> float:
    """Apply framework-aware threshold multiplier (1.5x for known frameworks)."""
    base = threshold.decorator_density_limit
    framework_imports = [
        "dataclasses", "pydantic", "attr", "attrs",
        "sqlalchemy", "django", "flask", "fastapi",
        "typing_extensions",
    ]
    for imp in info.imported_modules:
        for fw in framework_imports:
            if fw in imp:
                return base * 1.5
    return base
