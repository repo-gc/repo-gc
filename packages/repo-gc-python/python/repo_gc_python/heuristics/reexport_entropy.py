"""Barrel file / re-export detection — mirrors Rust's reexport_entropy.rs.

Python-specific: barrel files are `__init__.py` files that re-export symbols
via `from .module import Name` and/or declare `__all__`.
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
    limit = threshold.reexport_limit

    # Count re-exports: top-level imports in __init__.py (or any file with __all__)
    # that bring names into scope for re-export
    reexport_count = len(info.import_leaf_names) if info.is_init_file else 0
    total_items = reexport_count
    has_wildcard = info.has_star_imports
    has_all = info.all_export is not None

    # A non-init file with __all__ is also a form of deliberate re-export surface
    if has_all and not info.is_init_file:
        total_items = len(info.all_export) if info.all_export else 0
        reexport_count = 1  # one __all__ declaration

    if reexport_count < 3 and total_items < limit and not has_all:
        return None

    severity = Severity.Low
    if total_items >= limit * 3 or has_wildcard:
        severity = Severity.High
    elif total_items >= limit:
        severity = Severity.Medium

    reasons = [
        f"{reexport_count} re-export declarations"
        if info.is_init_file
        else f"__all__ with {total_items} symbols",
        f"{total_items} total re-exported symbols",
    ]
    if has_wildcard:
        reasons.append("Contains wildcard re-exports (from x import *)")
    if has_all:
        reasons.append(f"Declares __all__ ({len(info.all_export or [])} symbols)")

    return Finding(
        id=f"re-{counter:03d}",
        kind=FindingKind.ReexportEntropy,
        severity=severity,
        confidence=0.8,
        path=str(pfile.relative_path),
        summary=(
            f"Re-export chain — {total_items} symbols via imports, "
            f"agents traverse multiple files to resolve each import"
        ),
        reasons=reasons,
        evidence=[
            f"pub_use_count: {reexport_count}",
            f"total_reexported_items: {total_items}",
            f"has_wildcard: {str(has_wildcard).lower()}",
        ],
        suggested_next_step=(
            f"Flatten the re-export chain in {pfile.relative_path} — "
            f"barrel files degrade LLM path resolution"
        ),
        estimated_tokens=None,
    )
