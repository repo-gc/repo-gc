"""Unreferenced module detection — mirrors Rust's dead_weight.rs."""

from pathlib import Path

from ..discovery import PythonFile
from ..graph import ImportGraph
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity
from .context_bombs import estimate_tokens


def analyze_orphaned_files(
    pfiles: list[PythonFile],
    infos: list[FileInfo],
    graph: ImportGraph,
    counter: int,
) -> list[Finding]:
    # Build a set of all module paths referenced by any file's imports.
    # Include all prefixes so "mypackage.foo.bar" also marks "mypackage.foo".
    referenced: set[str] = set()

    for info in infos:
        for mod_path in info.imported_modules:
            parts = mod_path.split(".")
            for length in range(1, len(parts) + 1):
                referenced.add(".".join(parts[:length]))

    # Build path -> info lookup
    info_by_path: dict[Path, FileInfo] = {}
    for info in infos:
        info_by_path[info.path] = info

    findings: list[Finding] = []

    for pfile in pfiles:
        # Entry points are never orphaned
        info = info_by_path.get(pfile.path)
        if info is not None and (info.is_entry_point or info.is_init_file):
            continue

        # Fallback: check the stem
        stem = pfile.path.stem
        if stem in ("__init__", "__main__", "conftest"):
            continue

        module_path = info.module_path if info is not None else f"{pfile.package_name}.{stem}"

        # Check if this module or any prefix is referenced
        parts = module_path.split(".")
        is_referenced = module_path in referenced or any(
            ".".join(parts[:length]) in referenced for length in range(1, len(parts))
        )

        if not is_referenced:
            # Skip files with <10 lines (empty/near-empty files are not meaningful dead weight)
            if pfile.line_count < 10:
                continue

            # Check import-graph fan-in: if any other file imports this one, it is not orphaned
            if graph.fan_in.get(pfile.path, 0) > 0:
                continue

            severity = (
                Severity.Critical
                if pfile.line_count >= 1000
                else Severity.High
                if pfile.line_count >= 500
                else Severity.Medium
            )

            counter += 1
            findings.append(
                Finding(
                    id=f"dw-{counter:03d}",
                    kind=FindingKind.DeadWeight,
                    severity=severity,
                    confidence=0.6,
                    path=str(pfile.relative_path),
                    summary="",
                    reasons=[],
                    evidence=[
                        f"module_path: {module_path}",
                        f"line_count: {pfile.line_count}",
                        f"stem: {stem}",
                        f"estimated_tokens: {estimate_tokens(pfile.size_bytes)}",
                    ],
                    suggested_next_step="",
                    estimated_tokens=estimate_tokens(pfile.size_bytes),
                )
            )

    return findings
