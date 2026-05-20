"""Integration test — mirrors Rust's integration_test.rs.

Creates a temporary Python project with pyproject.toml and source files
that trigger each heuristic, runs the CLI, and validates JSON output.
"""

import json
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path


def _write(path: Path, content: str) -> None:
    path.write_text(textwrap.dedent(content))


def _run_scan(project_dir: Path, threshold: str = "normal") -> dict:
    """Run repo-gc-python scan on a project directory and return parsed JSON."""
    # cwd is the python/ directory containing repo_gc_python/
    tests_dir = Path(__file__).resolve().parent  # .../python/tests/
    py_dir = tests_dir.parent  # .../python/
    env = {**__import__("os").environ, "PYTHONPATH": str(py_dir)}
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "repo_gc_python",
            "scan",
            "--path",
            str(project_dir),
            "--format",
            "json",
            "--threshold",
            threshold,
        ],
        cwd=str(py_dir),
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise RuntimeError(f"CLI failed: {result.stderr}")
    return json.loads(result.stdout)


def test_json_output_has_expected_fields():
    """Verify JSON output has the required top-level fields."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        # Small file — no findings expected
        (root / "testproj").mkdir()
        _write(root / "testproj" / "__init__.py", "__version__ = '0.1.0'\n")
        _write(root / "testproj" / "mod.py", "def foo():\n    return 42\n")

        data = _run_scan(root)
        assert "findings" in data
        assert "global_score" in data
        assert "ai_friction_score" in data["global_score"]
        assert "files_analyzed" in data
        assert "files_skipped" in data


def test_context_bomb_detected():
    """A file exceeding the line count limit is flagged."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        pkg = root / "testproj"
        pkg.mkdir()
        _write(pkg / "__init__.py", "")

        # ~30 functions with padding: 30 * 17 lines = ~510 lines (>500 limit)
        funcs = []
        for i in range(30):
            funcs.append(f"def func_{i:03d}():")
            funcs.extend(f"    x{j} = {j}" for j in range(15))
            funcs.append("    return x1")
            funcs.append("")
        _write(pkg / "big.py", "\n".join(funcs))

        data = _run_scan(root)
        cb = [f for f in data["findings"] if f["kind"] == "ContextBomb"]
        assert len(cb) >= 1, f"Expected at least one ContextBomb, got {len(cb)}"


def test_dead_weight_detected():
    """An unreferenced module is flagged as dead weight."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        pkg = root / "testproj"
        pkg.mkdir()
        _write(pkg / "__init__.py", "")
        # orphan.py is never imported
        _write(pkg / "orphan.py", "def unused():\n    pass\n\nclass Orphaned:\n    pass\n")

        data = _run_scan(root)
        dw = [f for f in data["findings"] if f["kind"] == "DeadWeight"]
        assert len(dw) >= 1, f"Expected at least one DeadWeight, got {len(dw)}"


def test_reexport_entropy_detected():
    """A barrel __init__.py with many re-exports is flagged."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        pkg = root / "testproj"
        pkg.mkdir()
        # __init__.py with many imports (barrel file)
        _write(pkg / "__init__.py", """\
            from .a import A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11
            from .b import B1, B2, B3, B4, B5
        """)
        _write(pkg / "a.py", "A1=A2=A3=A4=A5=A6=A7=A8=A9=A10=A11=1\n")
        _write(pkg / "b.py", "B1=B2=B3=B4=B5=1\n")

        data = _run_scan(root)
        re = [f for f in data["findings"] if f["kind"] == "ReexportEntropy"]
        assert len(re) >= 1, f"Expected at least one ReexportEntropy, got {len(re)}"


def test_empty_dir_returns_zero_findings():
    """An empty project directory returns zero findings."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        data = _run_scan(root)
        assert data["findings"] == []
        assert data["files_analyzed"] == 0


def test_files_skipped_on_syntax_error():
    """Files with syntax errors are skipped, not crashed."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        pkg = root / "testproj"
        pkg.mkdir()
        _write(pkg / "__init__.py", "# valid\n")
        _write(pkg / "broken.py", "def oops(\n    # unclosed paren\n")

        data = _run_scan(root)
        assert data["files_skipped"] >= 1


def test_strict_threshold_finds_at_least_as_many_as_relaxed():
    """Strict threshold should find >= findings compared to relaxed."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write(root / "pyproject.toml", """\
            [build-system]
            requires = ["setuptools"]
            build-backend = "setuptools.build_meta"
            [project]
            name = "testproj"
            version = "0.1.0"
        """)
        pkg = root / "testproj"
        pkg.mkdir()
        _write(pkg / "__init__.py", "")
        # Create a file that straddles the strict/relaxed line limit
        lines = ["def func_{:03d}():".format(i) for i in range(20)]
        body = "\n".join(f"{name}\n    x = 1\n    return x\n" for name in lines)
        _write(pkg / "medium.py", body)

        strict_data = _run_scan(root, "strict")
        relaxed_data = _run_scan(root, "relaxed")
        assert len(strict_data["findings"]) >= len(relaxed_data["findings"])
