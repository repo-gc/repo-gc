"""AST-based Python file parser for repo-gc-python.

Mirrors Rust's parsing/extractor.rs.
Collects structural information from .py files for the import graph and heuristics.
"""

from __future__ import annotations

import ast
import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

logger = logging.getLogger("repo-gc-python")


@dataclass
class FileInfo:
    """Structural metadata extracted from a single .py file."""

    path: Path
    relative_path: Path
    package_name: str
    module_path: str  # e.g., "mypackage.foo.bar"
    function_count: int = 0
    public_function_count: int = 0  # top-level functions (all top-level defs in Python)
    class_count: int = 0
    function_bodies: list[tuple[str, str, str]] = field(default_factory=list)  # (name, raw_body, normalized_body)
    imported_modules: list[str] = field(default_factory=list)  # resolved module path strings
    import_leaf_names: list[str] = field(default_factory=list)  # final names brought into scope
    all_identifiers: set[str] = field(default_factory=set)  # all names referenced in file body
    is_init_file: bool = False  # True for __init__.py
    is_entry_point: bool = False
    has_star_imports: bool = False  # contains `from x import *`
    all_export: Optional[list[str]] = None  # __all__ declaration if present


# ---------------------------------------------------------------------------
# Module path computation
# ---------------------------------------------------------------------------


def module_path_from_relative(rel_path: Path, package_name: str) -> str:
    """Compute the fully qualified Python module path from a relative file path.

    Handles common Python source layouts:

    Examples (with package_name="mypackage"):
      src/foo/bar.py        ->  mypackage.foo.bar
      src/foo/__init__.py   ->  mypackage.foo
      mypackage/foo/bar.py  ->  mypackage.foo.bar   (package root dir auto-detected)
      mypackage/__init__.py ->  mypackage
    """
    parts = list(rel_path.parts)

    # Strip leading "src" directory (common Python source-layout convention)
    if parts and parts[0] == "src":
        parts = parts[1:]

    # Strip .py extension
    if parts and parts[-1].endswith(".py"):
        parts[-1] = parts[-1][:-3]

    # __init__.py represents the package itself -- drop it from the path
    if parts and parts[-1] == "__init__":
        parts = parts[:-1]

    # Avoid double-prefixing when the path already contains the package root
    if parts and parts[0] == package_name:
        parts = parts[1:]

    module_part = ".".join(parts) if parts else ""

    if module_part:
        return f"{package_name}.{module_part}"
    return package_name


# ---------------------------------------------------------------------------
# Import resolution helpers
# ---------------------------------------------------------------------------


def _resolve_import(
    level: int,
    module_name: str | None,
    current_module_path: str,
    is_init: bool,
) -> str | None:
    """Resolve a (possibly relative) import to an absolute module path string.

    Parameters
    ----------
    level :
        Number of leading dots in a ``from`` import (0 = absolute).
    module_name :
        The module part after the dots (e.g. ``"utils"`` in ``from ..utils import X``).
        ``None`` when the import has only dots (e.g. ``from . import foo``).
    current_module_path :
        Fully qualified module path of the *source* file.
    is_init :
        Whether the source file is an ``__init__.py`` (affects relative resolution).

    Returns
    -------
    The absolute module path being imported from, or ``None`` if no separate
    module is referenced (e.g. ``from . import name``).
    """
    # Absolute import
    if level == 0:
        return module_name  # may be None

    # Relative import with no explicit module -- the names are imported
    # from the package namespace itself, not from a separate module file.
    if level > 0 and not module_name:
        return None

    # Compute __package__-equivalent for the source module.
    # For a regular module __package__ is the parent; for __init__ it is the
    # module path itself (since __init__.py IS the package).
    parts = current_module_path.split(".")
    if is_init:
        package_parts = parts
    else:
        package_parts = parts[:-1] if len(parts) > 1 else []

    if not package_parts:
        return None

    # Go up (level - 1) from the package, then append the submodule.
    parent_depth = max(0, level - 1)
    if parent_depth > len(package_parts):
        return None

    base = package_parts[: len(package_parts) - parent_depth]
    base.append(module_name)
    return ".".join(base)


# ---------------------------------------------------------------------------
# Function body normalization
# ---------------------------------------------------------------------------


def _normalize_body(body: list[ast.stmt]) -> tuple[str, str]:
    """Normalize a function body for duplication detection.

    Returns ``(raw_body, normalized_body)`` where *raw_body* is the
    ``ast.unparse`` output (valid Python, may contain whitespace) and
    *normalized_body* is the same with all whitespace stripped.
    """
    module = ast.Module(body=body, type_ignores=[])
    text = ast.unparse(module)
    return text, "".join(text.split())


# ---------------------------------------------------------------------------
# AST visitors
# ---------------------------------------------------------------------------


class _FileVisitor(ast.NodeVisitor):
    """First-pass AST visitor collecting structural information.

    Collects functions (including async), classes, imports, ``__all__``
    exports, ``__name__ == "__main__"`` guards, and star imports.

    Does **not** recurse into functions/methods/classes because the elements
    we care about (imports, ``__all__``, entry-point guards) are only
    meaningful at module level.  This avoids double-counting.
    """

    def __init__(self, path: Path, relative_path: Path, package_name: str, module_path: str) -> None:
        self.file_info = FileInfo(
            path=path,
            relative_path=relative_path,
            package_name=package_name,
            module_path=module_path,
            is_init_file=relative_path.name == "__init__.py",
        )
        self._has_main_guard = False
        self._has_future_annotations = False

    # -- functions ----------------------------------------------------------

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self.file_info.function_count += 1
        self.file_info.public_function_count += 1  # top-level = public in Python
        raw_body, norm_body = _normalize_body(node.body)
        self.file_info.function_bodies.append((node.name, raw_body, norm_body))

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        self.file_info.function_count += 1
        self.file_info.public_function_count += 1
        raw_body, norm_body = _normalize_body(node.body)
        self.file_info.function_bodies.append((node.name, raw_body, norm_body))

    # -- classes ------------------------------------------------------------

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        self.file_info.class_count += 1
        # Do NOT recurse -- methods inside classes are not top-level functions.

    # -- imports ------------------------------------------------------------

    def visit_Import(self, node: ast.Import) -> None:
        for alias in node.names:
            self.file_info.imported_modules.append(alias.name)
            leaf = alias.asname if alias.asname else alias.name.split(".")[0]
            self.file_info.import_leaf_names.append(leaf)

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        # Detect ``from __future__ import annotations`` (PEP 563)
        if node.module == "__future__":
            for alias in node.names:
                if alias.name == "annotations":
                    self._has_future_annotations = True

        # Resolve the module path being imported from
        resolved = _resolve_import(
            node.level,
            node.module,
            self.file_info.module_path,
            self.file_info.is_init_file,
        )
        if resolved:
            self.file_info.imported_modules.append(resolved)

        # Collect leaf names and detect star imports
        for alias in node.names:
            if alias.name == "*":
                self.file_info.has_star_imports = True
            else:
                leaf = alias.asname if alias.asname else alias.name
                self.file_info.import_leaf_names.append(leaf)

    # -- __all__ export -----------------------------------------------------

    def visit_Assign(self, node: ast.Assign) -> None:
        # Detect __all__ = [...] at module level
        if (
            len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
            and node.targets[0].id == "__all__"
        ):
            if isinstance(node.value, (ast.List, ast.Tuple)):
                exports: list[str] = []
                for elt in node.value.elts:
                    if isinstance(elt, ast.Constant) and isinstance(elt.value, str):
                        exports.append(elt.value)
                if exports:
                    self.file_info.all_export = exports
        # generic_visit is NOT called -- we only care about module-level assigns
        # and our overrides of FunctionDef/ClassDef already stop recursion.

    # -- __name__ == "__main__" guard ---------------------------------------

    def visit_If(self, node: ast.If) -> None:
        if (
            isinstance(node.test, ast.Compare)
            and isinstance(node.test.left, ast.Name)
            and node.test.left.id == "__name__"
            and len(node.test.ops) == 1
            and isinstance(node.test.ops[0], ast.Eq)
            and len(node.test.comparators) == 1
            and isinstance(node.test.comparators[0], ast.Constant)
            and node.test.comparators[0].value == "__main__"
        ):
            self._has_main_guard = True
        # generic_visit is NOT called for the same reason as visit_Assign.

    # -- query helpers ------------------------------------------------------

    def has_future_annotations(self) -> bool:
        return self._has_future_annotations


class _IdentCollector(ast.NodeVisitor):
    """Second-pass AST visitor collecting all referenced identifiers.

    Excludes import statement aliases.  Handles several AST edge cases that
    a plain ``ast.walk()`` would miss:

    * PEP 563 (``from __future__ import annotations``) -- annotations become
      ``Constant`` string nodes rather than ``Name`` nodes.  The collector
      parses annotation-position strings to extract identifiers.
    * PEP 695 type-parameter syntax (``TypeVar``, ``ParamSpec``,
      ``TypeVarTuple``) -- Python 3.12+.
    * ``match`` statement patterns (``MatchAs``).
    * ``except ... as e`` handler names (``ExceptHandler.name``).
    """

    def __init__(self, has_future_annotations: bool = False) -> None:
        self.idents: set[str] = set()
        self._has_future_annotations = has_future_annotations

    # -- Name nodes (all contexts: Load, Store, Del) ------------------------

    def visit_Name(self, node: ast.Name) -> None:
        self.idents.add(node.id)
        self.generic_visit(node)

    # -- Skip imports entirely ---------------------------------------------

    def visit_Import(self, node: ast.Import) -> None:
        pass

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        pass

    # -- PEP 695 type-parameter nodes (Python 3.12+) -----------------------

    def visit_TypeVar(self, node: ast.AST) -> None:
        name: str | None = getattr(node, "name", None)
        if name:
            self.idents.add(name)
        self.generic_visit(node)

    def visit_ParamSpec(self, node: ast.AST) -> None:
        name = getattr(node, "name", None)
        if name:
            self.idents.add(name)
        self.generic_visit(node)

    def visit_TypeVarTuple(self, node: ast.AST) -> None:
        name = getattr(node, "name", None)
        if name:
            self.idents.add(name)
        self.generic_visit(node)

    # -- match-statement pattern bindings (Python 3.10+) -------------------

    def visit_MatchAs(self, node: ast.MatchAs) -> None:
        if node.name:
            self.idents.add(node.name)
        self.generic_visit(node)

    # -- except ... as e ---------------------------------------------------

    def visit_ExceptHandler(self, node: ast.ExceptHandler) -> None:
        if node.name:
            self.idents.add(node.name)
        self.generic_visit(node)

    # -- PEP 563 annotation string extraction ------------------------------

    def _extract_names_from_type_str(self, s: str) -> None:
        """Parse *s* as a type expression and collect referenced ``Name`` nodes."""
        try:
            tree = ast.parse(s, mode="eval")
            for n in ast.walk(tree):
                if isinstance(n, ast.Name):
                    self.idents.add(n.id)
        except SyntaxError:
            pass

    def _check_annotation(self, ann: ast.AST | None) -> None:
        """If *ann* is a Constant string (PEP 563), extract identifiers from it."""
        if isinstance(ann, ast.Constant) and isinstance(ann.value, str):
            self._extract_names_from_type_str(ann.value)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        if self._has_future_annotations:
            self._check_annotation(node.returns)
            for arg in node.args.args + node.args.kwonlyargs + node.args.posonlyargs:
                self._check_annotation(arg.annotation)
            if node.args.vararg:
                self._check_annotation(node.args.vararg.annotation)
            if node.args.kwarg:
                self._check_annotation(node.args.kwarg.annotation)
        self.generic_visit(node)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        if self._has_future_annotations:
            self._check_annotation(node.returns)
            for arg in node.args.args + node.args.kwonlyargs + node.args.posonlyargs:
                self._check_annotation(arg.annotation)
            if node.args.vararg:
                self._check_annotation(node.args.vararg.annotation)
            if node.args.kwarg:
                self._check_annotation(node.args.kwarg.annotation)
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        if self._has_future_annotations:
            self._check_annotation(node.annotation)
        self.generic_visit(node)


# ---------------------------------------------------------------------------
# Entry-point detection
# ---------------------------------------------------------------------------


def _is_entry_point(info: FileInfo, has_main_guard: bool) -> bool:
    """Determine whether *info* describes a project entry point."""
    if info.is_init_file:
        return True
    if info.path.name == "__main__.py":
        return True
    if has_main_guard:
        return True
    # Common entry-point basenames
    if info.path.name in ("conftest.py", "manage.py", "app.py", "cli.py"):
        return True
    return False


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def extract_file_info(
    path: Path,
    package_name: str,
    workspace_root: Path,
) -> Optional[FileInfo]:
    """Parse a Python file and extract structural information.

    Returns ``None`` if the file cannot be parsed (syntax errors, null bytes,
    deeply nested source, or other unrecoverable errors).
    """
    # -- read & parse -------------------------------------------------------
    try:
        with open(path, "rb") as f:
            raw = f.read()

        if b"\0" in raw:
            logger.debug("Skipping %s: null bytes in source", path)
            return None

        source = raw.decode("utf-8")
        tree = ast.parse(source, filename=str(path))
    except SyntaxError:
        logger.debug("Skipping %s: syntax error", path)
        return None
    except ValueError:
        logger.debug("Skipping %s: null bytes or encoding error", path)
        return None
    except RecursionError:
        logger.debug("Skipping %s: deeply nested source", path)
        return None
    except Exception:
        logger.debug("Skipping %s: unexpected parse error", path, exc_info=True)
        return None

    # -- relative path & module path ----------------------------------------
    try:
        relative_path = path.relative_to(workspace_root)
    except ValueError:
        # path is not under workspace_root (e.g. symlink escape)
        relative_path = path

    module_path = module_path_from_relative(relative_path, package_name)

    # -- first pass: structural visitor -------------------------------------
    try:
        visitor = _FileVisitor(path, relative_path, package_name, module_path)
        visitor.visit(tree)
    except RecursionError:
        logger.debug("Skipping %s: recursion during AST visit", path)
        return None

    info = visitor.file_info

    # -- second pass: identifier collection ---------------------------------
    collector = _IdentCollector(has_future_annotations=visitor.has_future_annotations())
    collector.visit(tree)
    info.all_identifiers = collector.idents

    # -- entry point detection ----------------------------------------------
    info.is_entry_point = _is_entry_point(info, visitor._has_main_guard)

    return info
