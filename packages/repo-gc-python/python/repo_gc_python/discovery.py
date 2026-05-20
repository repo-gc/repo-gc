"""Python project discovery and file enumeration for repo-gc-python.

Implements a 3-pass discovery algorithm:
  Pass 1 — Find manifest (pyproject.toml, setup.py, setup.cfg)
  Pass 2 — Find packages from source roots
  Pass 3 — Heuristic fallback when no manifest found
"""

from __future__ import annotations

import configparser
import os
import re
from dataclasses import dataclass, field
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore[no-redef]


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class PackageInfo:
    """A discovered Python package."""

    name: str
    manifest_path: Path  # Path to the pyproject.toml / setup.py / setup.cfg
    source_roots: list[Path] = field(default_factory=list)


@dataclass
class WorkspaceInfo:
    """Top-level workspace / project discovery result."""

    root: Path
    packages: list[PackageInfo] = field(default_factory=list)
    is_workspace: bool = False


@dataclass
class PythonFile:
    """A single .py file discovered under a package source root."""

    path: Path
    relative_path: Path
    package_name: str
    size_bytes: int
    line_count: int


# ---------------------------------------------------------------------------
# Directory / file exclusion
# ---------------------------------------------------------------------------

EXCLUDE_DIRS: frozenset[str] = frozenset({
    "__pycache__",
    ".git",
    "venv",
    ".venv",
    ".tox",
    ".eggs",
    "dist",
    "build",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "node_modules",
})

EXCLUDE_DIR_SUFFIXES: tuple[str, ...] = (".egg-info",)


def _is_excluded_dir(dirname: str) -> bool:
    return dirname in EXCLUDE_DIRS or dirname.endswith(EXCLUDE_DIR_SUFFIXES)


def _is_test_path(rel_path: Path) -> bool:
    """Return True when *rel_path* looks like a test file."""
    parts = rel_path.parts
    if "tests" in parts or "test" in parts:
        return True
    name = rel_path.name
    if name.startswith("test_") or name.endswith("_test.py"):
        return True
    if name == "conftest.py":
        return True
    return False


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _count_lines(path: Path) -> int:
    """Count lines in a text file efficiently (binary read, no decode)."""
    try:
        with open(path, "rb") as f:
            return sum(1 for _ in f)
    except OSError:
        return 0


def _normalize_pkg_name(rel: Path) -> str:
    """Convert a relative path like ``repo_gc_python/reporters`` to
    ``repo_gc_python.reporters`` (OS-agnostic)."""
    return str(rel).replace(os.sep, ".")


# ---------------------------------------------------------------------------
# Pass 1 — Manifest discovery
# ---------------------------------------------------------------------------


def _find_manifest(start: Path) -> Path | None:
    """Walk up from *start* looking for a recognised manifest file.

    Returns the first ``pyproject.toml``, ``setup.py`` or ``setup.cfg`` found,
    or ``None`` when none exists in any parent directory.
    """
    for parent in [start] + list(start.parents):
        pyproject = parent / "pyproject.toml"
        if pyproject.is_file():
            return pyproject
        for name in ("setup.py", "setup.cfg"):
            candidate = parent / name
            if candidate.is_file():
                return candidate
    return None


def _detect_backend(data: dict) -> str | None:
    """Identify the build backend from a parsed ``pyproject.toml`` dict."""
    backend = data.get("build-system", {}).get("build-backend", "")
    if "setuptools" in backend:
        return "setuptools"
    if "poetry" in backend:
        return "poetry"
    if "hatchling" in backend:
        return "hatchling"
    if "pdm" in backend:
        return "pdm"
    if "flit" in backend:
        return "flit"
    if "maturin" in backend:
        return "maturin"
    return None


def _extract_source_roots(
    manifest_path: Path, manifest_dir: Path
) -> list[Path]:
    """Dispatch to the right parser based on manifest filename."""
    if manifest_path.name == "pyproject.toml":
        return _extract_source_roots_pyproject(manifest_path, manifest_dir)
    if manifest_path.name == "setup.cfg":
        return _extract_source_roots_setup_cfg(manifest_path, manifest_dir)
    if manifest_path.name == "setup.py":
        return _extract_source_roots_setup_py(manifest_path, manifest_dir)
    return [manifest_dir]


def _extract_source_roots_pyproject(
    manifest_path: Path, manifest_dir: Path
) -> list[Path]:
    """Parse ``pyproject.toml`` for source roots, keyed by build backend."""
    try:
        with open(manifest_path, "rb") as f:
            data = tomllib.load(f)
    except Exception:
        return [manifest_dir]

    backend = _detect_backend(data)
    if backend is None:
        return [manifest_dir]

    tool = data.get("tool", {})

    # -- setuptools ---------------------------------------------------------
    if backend == "setuptools":
        find_where = (
            tool.get("setuptools", {})
            .get("packages", {})
            .get("find", {})
            .get("where", ["."])
        )
        return [manifest_dir / w for w in find_where]

    # -- poetry -------------------------------------------------------------
    if backend == "poetry":
        pkgs = tool.get("poetry", {}).get("packages", [])
        if not pkgs:
            return [manifest_dir]
        seen: list[Path] = []
        for entry in pkgs:
            p = manifest_dir / entry.get("from", ".")
            if p not in seen:
                seen.append(p)
        return seen

    # -- hatchling ----------------------------------------------------------
    if backend == "hatchling":
        wheel = (
            tool.get("hatch", {})
            .get("build", {})
            .get("targets", {})
            .get("wheel", {})
        )
        sources: dict = wheel.get("sources") or {}
        if sources:
            # ``sources`` maps package-name → directory
            return list({manifest_dir / d for d in sources.values()})
        packages = wheel.get("packages")
        if packages:
            return [manifest_dir / p for p in packages]
        return [manifest_dir]

    # -- pdm ----------------------------------------------------------------
    if backend == "pdm":
        pkg_dir = tool.get("pdm", {}).get("build", {}).get("package-dir", ".")
        return [manifest_dir / pkg_dir]

    # -- flit ---------------------------------------------------------------
    if backend == "flit":
        module = tool.get("flit", {}).get("module", "")
        if not module:
            project = data.get("project", {})
            module = project.get("name", "").replace("-", "_").replace(".", "_")
        if module and (manifest_dir / "src" / module).is_dir():
            return [manifest_dir / "src"]
        return [manifest_dir]

    # -- maturin ------------------------------------------------------------
    if backend == "maturin":
        py_src = tool.get("maturin", {}).get("python-source", ".")
        return [manifest_dir / py_src]

    return [manifest_dir]


def _extract_source_roots_setup_cfg(
    manifest_path: Path, manifest_dir: Path
) -> list[Path]:
    """Parse ``setup.cfg`` to find ``options.packages.find.where``."""
    try:
        cfg = configparser.ConfigParser()
        cfg.read(manifest_path)

        # options.packages.find → where (the most common pattern)
        if cfg.has_section("options.packages.find"):
            where_raw = cfg.get("options.packages.find", "where", fallback=".")
            dirs = [w.strip() for w in where_raw.split("\n") if w.strip()]
            return [manifest_dir / d for d in dirs] if dirs else [manifest_dir]

        # options → package_dir dict
        if cfg.has_section("options.package_dir"):
            dirlist = [manifest_dir / v for _, v in cfg.items("options.package_dir")]
            return dirlist if dirlist else [manifest_dir]

        # options → packages = find:  (setuptools-style)
        if cfg.has_option("options", "packages"):
            val = cfg.get("options", "packages", fallback="").strip()
            if val.startswith("find:") or val.startswith("find_namespace:"):
                pass  # fall through to default below
    except Exception:
        pass
    return [manifest_dir]


def _extract_source_roots_setup_py(
    manifest_path: Path, manifest_dir: Path
) -> list[Path]:
    """Parse ``setup.py`` with regex.  Does *not* execute the file."""
    try:
        raw = manifest_path.read_text(encoding="utf-8")
    except Exception:
        return [manifest_dir]

    # find_packages(where=["a", "b"])
    m = re.search(
        r"""find_packages\s*\(\s*where\s*=\s*\[([^\]]*)\]""", raw
    )
    if m:
        items = re.findall(r"""["']([^"']+)["']""", m.group(1))
        return [manifest_dir / it for it in items] if items else [manifest_dir]

    # find_packages(where="single")
    m = re.search(
        r"""find_packages\s*\(\s*where\s*=\s*["']([^"']+)["']""", raw
    )
    if m:
        return [manifest_dir / m.group(1)]

    # package_dir = {"": "src", ...}
    m = re.search(r"""package_dir\s*=\s*\{(.+?)\}""", raw)
    if m:
        pairs = re.findall(
            r"""["']([^"']*)["']\s*:\s*["']([^"']+)["']""", m.group(1)
        )
        dirs = [d[1] for d in pairs]
        return [manifest_dir / d for d in dirs] if dirs else [manifest_dir]

    return [manifest_dir]


# ---------------------------------------------------------------------------
# Pass 2 — Package discovery
# ---------------------------------------------------------------------------


def _find_packages_in_source_roots(
    source_roots: list[Path],
    manifest_dir: Path,
    include_namespace_packages: bool = False,
) -> list[PackageInfo]:
    """Walk *source_roots* and return every Python package found.

    Regular packages are directories that contain ``__init__.py``.
    Namespace packages (PEP 420) — directories without ``__init__.py`` that
    contain sub-packages — are returned only when
    *include_namespace_packages* is ``True`` (default ``False``).

    Each returned ``PackageInfo`` carries *its own* directory as the sole
    source root, so ``enumerate_python_files`` can walk precisely the files
    belonging to that package.
    """
    found: list[PackageInfo] = []

    for sroot in source_roots:
        resolved = sroot.resolve()
        if not resolved.is_dir():
            continue

        for dirpath, dirnames, filenames in os.walk(resolved):
            # Prune excluded subtrees
            dirnames[:] = [
                d for d in dirnames if not _is_excluded_dir(d)
            ]

            current = Path(dirpath)
            has_init = "__init__.py" in filenames

            if has_init:
                pkg_rel = current.relative_to(resolved)
                pkg_name = (
                    resolved.name
                    if str(pkg_rel) == "."
                    else _normalize_pkg_name(pkg_rel)
                )
                found.append(
                    PackageInfo(
                        name=pkg_name,
                        manifest_path=manifest_dir,
                        source_roots=[current],
                    )
                )
            elif include_namespace_packages and current != resolved:
                # PEP 420 namespace: a directory *without* __init__.py that
                # contains at least one sub-directory with __init__.py
                has_sub = any(
                    (current / d / "__init__.py").is_file()
                    for d in dirnames
                    if not _is_excluded_dir(d)
                )
                if has_sub:
                    pkg_rel = current.relative_to(resolved)
                    pkg_name = _normalize_pkg_name(pkg_rel)
                    found.append(
                        PackageInfo(
                            name=pkg_name,
                            manifest_path=manifest_dir,
                            source_roots=[current],
                        )
                    )

    # Deduplicate by name (in case the same package is reachable from
    # multiple roots).
    seen: set[str] = set()
    unique: list[PackageInfo] = []
    for p in found:
        if p.name not in seen:
            seen.add(p.name)
            unique.append(p)
    return unique


def _find_packages_heuristic(
    search_dir: Path, manifest_dir: Path
) -> list[PackageInfo]:
    """Fallback when no manifest is found (Pass 3)."""
    # 1) src/<something>/__init__.py
    src_dir = search_dir / "src"
    if src_dir.is_dir():
        for child in src_dir.iterdir():
            if child.is_dir() and (child / "__init__.py").is_file():
                return [
                    PackageInfo(
                        name=child.name,
                        manifest_path=manifest_dir,
                        source_roots=[child],
                    )
                ]

    # 2) search_dir itself contains __init__.py
    if (search_dir / "__init__.py").is_file():
        return [
            PackageInfo(
                name=search_dir.name,
                manifest_path=manifest_dir,
                source_roots=[search_dir],
            )
        ]

    # 3) Flat scripts: any .py file at the top level
    try:
        has_py = any(f.suffix == ".py" for f in search_dir.iterdir())
    except OSError:
        has_py = False
    if has_py:
        return [
            PackageInfo(
                name=search_dir.name,
                manifest_path=manifest_dir,
                source_roots=[search_dir],
            )
        ]

    return []


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def discover_workspace(path: Path) -> WorkspaceInfo:
    """Run the 3-pass discovery algorithm.

    Pass 1 — Locate and parse a manifest (``pyproject.toml``, ``setup.py``
    or ``setup.cfg``) by walking up from *path*.

    Pass 2 — Walk the source roots declared in the manifest and discover
    Python packages.

    Pass 3 — When no manifest is found, apply heuristics to locate
    packages.
    """
    target = path.resolve()

    # -- Pass 1 ------------------------------------------------------------
    manifest = _find_manifest(target)

    if manifest is not None:
        manifest_dir = manifest.parent.resolve()
        source_roots = _extract_source_roots(manifest, manifest_dir)

        # -- Pass 2 --------------------------------------------------------
        packages = _find_packages_in_source_roots(source_roots, manifest_dir)

        return WorkspaceInfo(
            root=manifest_dir,
            packages=packages,
            is_workspace=len(packages) > 1,
        )

    # -- Pass 3 ------------------------------------------------------------
    packages = _find_packages_heuristic(target, target)
    return WorkspaceInfo(
        root=target,
        packages=packages,
        is_workspace=False,
    )


def enumerate_python_files(
    packages: list[PackageInfo],
    workspace_root: Path,
    include_tests: bool = False,
) -> list[PythonFile]:
    """Walk every *package* source root and collect ``.py`` files.

    Files are deduplicated by canonical (resolved) path.
    When *include_tests* is ``False`` (the default), files matching test
    naming or directory conventions are excluded.
    """
    seen: set[Path] = set()
    result: list[PythonFile] = []
    workspace_root_resolved = workspace_root.resolve()

    for pkg in packages:
        for src_root in pkg.source_roots:
            resolved_root = src_root.resolve()
            if not resolved_root.is_dir():
                continue

            for dirpath, dirnames, filenames in os.walk(resolved_root):
                dirnames[:] = [
                    d for d in dirnames if not _is_excluded_dir(d)
                ]

                current = Path(dirpath)

                for filename in filenames:
                    if not filename.endswith(".py"):
                        continue

                    file_path = current / filename
                    canonical = file_path.resolve()

                    if canonical in seen:
                        continue
                    seen.add(canonical)

                    rel_path = canonical.relative_to(workspace_root_resolved)

                    if not include_tests and _is_test_path(rel_path):
                        continue

                    try:
                        st = canonical.stat()
                        size_bytes = st.st_size
                    except OSError:
                        size_bytes = 0

                    result.append(
                        PythonFile(
                            path=canonical,
                            relative_path=rel_path,
                            package_name=pkg.name,
                            size_bytes=size_bytes,
                            line_count=_count_lines(canonical),
                        )
                    )

    return result
