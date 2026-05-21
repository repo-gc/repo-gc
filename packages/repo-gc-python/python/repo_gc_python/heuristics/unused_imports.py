"""Zombie import detection — mirrors Rust's unused_imports.rs.

Handles `from __future__ import annotations` (PEP 563) via the parsing module's
pre-processed identifier collection, which already parses annotation strings for
identifier names when the future import is active.
"""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity

MIN_UNUSED = 2

# Names commonly referenced by compile-time transforms rather than user code.
# Empty for Python — the language has no JSX-like implicit usage.
COMPILER_NAMES: frozenset[str] = frozenset()


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    _threshold: Threshold,
    counter: int,
) -> Finding | None:
    unused = [
        name
        for name in info.import_leaf_names
        if name not in COMPILER_NAMES and name not in info.all_identifiers
    ]

    if len(unused) < MIN_UNUSED:
        return None

    severity = Severity.Medium if len(unused) >= 5 else Severity.Low

    preview = unused[:5]
    return Finding(
        id=f"ui-{counter:03d}",
        kind=FindingKind.UnusedImport,
        severity=severity,
        confidence=0.65,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"unused_count: {len(unused)}",
            f"preview: {', '.join(preview)}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
