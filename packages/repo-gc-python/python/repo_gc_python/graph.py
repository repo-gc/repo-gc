"""Import dependency graph — mirrors Rust's graph/import_graph.rs.

Builds fan-in/fan-out maps from FileInfo imports using longest-prefix matching
with path-segment verification and stdlib module exclusion.
"""

from __future__ import annotations

import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from .parsing import FileInfo

STDLIB_MODULES: frozenset[str] = getattr(sys, "stdlib_module_names", frozenset())


def _is_stdlib(module_path: str) -> bool:
    """Check whether a module path refers to the standard library."""
    top = module_path.split(".", 1)[0]
    return top in STDLIB_MODULES


def _file_tree(files: list[FileInfo]) -> set[str]:
    """Build a set of all known file stems and directories for segment verification.

    Returns dot-separated relative paths like {"mypackage.foo", "mypackage.foo.bar"}.
    """
    tree: set[str] = set()
    for f in files:
        parts = f.module_path.split(".")
        for i in range(1, len(parts) + 1):
            tree.add(".".join(parts[:i]))
    return tree


@dataclass
class ImportGraph:
    """Tracks how many files import each file (fan-in) and how many each imports (fan-out)."""

    fan_in: dict[Path, int] = field(default_factory=dict)
    fan_out: dict[Path, int] = field(default_factory=dict)

    @classmethod
    def build(cls, files: list[FileInfo], workspace_root: Path) -> ImportGraph:
        # Map module_path → canonical Path (deduplicated)
        module_to_path: dict[str, Path] = {}
        for f in files:
            existing = module_to_path.get(f.module_path)
            if existing is None:
                module_to_path[f.module_path] = f.path

        # All known file-level prefixes for segment verification
        tree = _file_tree(files)

        graph = cls(
            fan_in={f.path: 0 for f in files},
            fan_out={f.path: 0 for f in files},
        )

        for src in files:
            resolved_targets: set[Path] = set()

            for mod_path in src.imported_modules:
                # Skip stdlib
                if _is_stdlib(mod_path):
                    continue

                target = _resolve(mod_path, module_to_path, tree)
                if target is not None and target != src.path:
                    resolved_targets.add(target)

            graph.fan_out[src.path] = len(resolved_targets)
            for tgt in resolved_targets:
                graph.fan_in[tgt] += 1

        return graph


def _resolve(
    import_path: str,
    module_to_path: dict[str, Path],
    file_tree: set[str],
) -> Optional[Path]:
    """Resolve an absolute import path to a known file using longest-prefix matching.

    After matching a prefix, verifies that each remaining segment exists in
    the file tree (as ``<seg>.py`` or ``<seg>/__init__.py`` equivalent).
    This prevents false edges like ``import a.b.c`` resolving to ``a/__init__.py``
    when ``a/b/__init__.py`` does not exist.
    """
    parts = import_path.split(".")

    # Try longest-prefix match
    for length in range(len(parts), 0, -1):
        prefix = ".".join(parts[:length])
        if prefix in module_to_path:
            # Verify intermediate segments exist in the file tree
            if not _verify_chain(parts[:length], parts[length:], file_tree):
                continue
            return module_to_path[prefix]

    return None


def _verify_chain(
    matched_parts: list[str],
    remaining_parts: list[str],
    file_tree: set[str],
) -> bool:
    """Check that each remaining import segment corresponds to a real file or package.

    For ``import a.b.c.d`` with prefix ``a.b`` matched (so remaining = ['c', 'd']):
    we need ``a.b.c`` to exist in the file tree (as a file or directory).
    """
    current = list(matched_parts)
    for seg in remaining_parts:
        current.append(seg)
        candidate = ".".join(current)
        if candidate not in file_tree:
            return False
    return True


def resolve_relative(
    base_module: str,
    dot_count: int,
    target: str,
    module_to_path: dict[str, Path],
    file_tree: set[str],
) -> Optional[Path]:
    """Resolve a relative import like ``from . import foo`` or ``from ..bar import baz``.

    *dot_count* is the number of leading dots (1 for ``.``, 2 for ``..``, etc.).
    *target* is the module name after the dots (empty string for bare ``from . import``).
    """
    base_parts = base_module.split(".")
    # ``.`` strips nothing, ``..`` strips one, ``...`` strips two, etc.
    if dot_count > len(base_parts):
        return None  # Beyond root

    new_parts = base_parts[: len(base_parts) - dot_count + 1]
    if target:
        new_parts.append(target)
    full = ".".join(new_parts)

    return _resolve(full, module_to_path, file_tree)
