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


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    _threshold: Threshold,
    counter: int,
) -> Finding | None:
    unused = [name for name in info.import_leaf_names if name not in info.all_identifiers]

    if len(unused) < MIN_UNUSED:
        return None

    severity = Severity.Medium if len(unused) >= 5 else Severity.Low

    preview = unused[:5]
    return Finding(
        id=f"ui-{counter:03d}",
        kind=FindingKind.UnusedImport,
        severity=severity,
        confidence=0.65,  # Low confidence: dynamic patterns may reference names invisibly
        path=str(pfile.relative_path),
        summary=(
            f"Zombie imports — {len(unused)} unused names inflate token usage "
            f"in every context window: {', '.join(preview)}"
        ),
        reasons=[
            f"{len(unused)} imported names not referenced in file body; "
            f"dynamic patterns may cause false positives"
        ],
        evidence=[f"imported but unreferenced: {n}" for n in unused],
        suggested_next_step=(
            "Remove zombie imports to reduce token waste, or verify they are "
            "needed by dynamic usage patterns"
        ),
        estimated_tokens=None,
    )
