"""CLI argument parsing and pipeline orchestration.

Mirrors Rust's cli.rs + main.rs analyze() pipeline.
"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from . import __version__
from .types import Finding, Report, Severity

logger = logging.getLogger("repo-gc-python")

OUTPUT_FORMATS = ("text", "json", "md", "llm")
THRESHOLD_LEVELS = ("strict", "normal", "relaxed")


class Threshold:
    """Mirrors Rust's Threshold enum with limit accessors."""

    def __init__(self, level: str) -> None:
        self._level = level

    @property
    def line_count_limit(self) -> int:
        return {"strict": 300, "normal": 500, "relaxed": 1000}[self._level]

    @property
    def fan_in_limit(self) -> int:
        return {"strict": 5, "normal": 10, "relaxed": 20}[self._level]

    @property
    def fan_out_limit(self) -> int:
        return {"strict": 10, "normal": 15, "relaxed": 25}[self._level]

    @property
    def reexport_limit(self) -> int:
        return {"strict": 5, "normal": 10, "relaxed": 20}[self._level]

    @property
    def branch_density_limit(self) -> int:
        return {"strict": 8, "normal": 12, "relaxed": 18}[self._level]

    @property
    def nesting_depth_limit(self) -> int:
        return {"strict": 4, "normal": 6, "relaxed": 8}[self._level]

    @property
    def type_depth_limit(self) -> int:
        return {"strict": 3, "normal": 4, "relaxed": 5}[self._level]

    @property
    def comment_ratio_min(self) -> float:
        return {"strict": 0.03, "normal": 0.03, "relaxed": 0.01}[self._level]

    @property
    def comment_ratio_max(self) -> float:
        return {"strict": 0.20, "normal": 0.30, "relaxed": 0.50}[self._level]

    @property
    def decorator_density_limit(self) -> float:
        return {"strict": 0.33, "normal": 0.50, "relaxed": 0.75}[self._level]

    @property
    def empty_catch_limit(self) -> int:
        return {"strict": 1, "normal": 2, "relaxed": 3}[self._level]

    @property
    def dangerous_pattern_limit(self) -> int:
        return {"strict": 3, "normal": 5, "relaxed": 10}[self._level]

    @property
    def mutable_global_limit(self) -> int:
        return {"strict": 3, "normal": 5, "relaxed": 8}[self._level]

    @property
    def string_comparison_limit(self) -> int:
        return {"strict": 5, "normal": 10, "relaxed": 15}[self._level]

    @property
    def platform_conditional_limit(self) -> int:
        return {"strict": 3, "normal": 5, "relaxed": 10}[self._level]

    @property
    def import_domain_limit(self) -> int:
        return {"strict": 8, "normal": 10, "relaxed": 14}[self._level]

    @property
    def level(self) -> str:
        return self._level


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="repo-gc-python",
        description="Find patterns that waste tokens, confuse AI agents, and break AI-assisted edits",
    )
    subs = parser.add_subparsers(dest="command", required=True)

    # scan
    scan = subs.add_parser("scan", help="Scan for AI-context-wasting patterns")
    scan.add_argument("--path", default=".", type=Path, help="Target directory (default: .)")
    scan.add_argument("--format", default="text", choices=OUTPUT_FORMATS, help="Output format")
    scan.add_argument("--threshold", default="normal", choices=THRESHOLD_LEVELS, help="Sensitivity level")
    scan.add_argument("--include-tests", action="store_true", help="Include test files")
    scan.add_argument("--no-color", action="store_true", help="Disable colored output")
    scan.add_argument("--lang", default="python", help="Language (default: python)")

    # report
    report = subs.add_parser("report", help="Generate an AI Context Efficiency report")
    report.add_argument("--path", default=".", type=Path, help="Target directory")
    report.add_argument("--format", default="text", choices=OUTPUT_FORMATS, help="Output format")
    report.add_argument("--threshold", default="normal", choices=THRESHOLD_LEVELS, help="Sensitivity level")
    report.add_argument("--include-tests", action="store_true", help="Include test files")
    report.add_argument("--lang", default="python", help="Language")

    # json (shortcut for scan --format json)
    js = subs.add_parser("json", help="Emit findings as JSON (CI integration)")
    js.add_argument("--path", default=".", type=Path, help="Target directory")
    js.add_argument("--threshold", default="normal", choices=THRESHOLD_LEVELS, help="Sensitivity level")
    js.add_argument("--include-tests", action="store_true", help="Include test files")
    js.add_argument("--lang", default="python", help="Language")

    # explain
    explain = subs.add_parser("explain", help="Explain why a file degrades AI context efficiency")
    explain.add_argument("path", type=Path, help="File to explain")
    explain.add_argument("--root", default=".", type=Path, help="Project root")

    return parser


def analyze(path: Path, threshold: Threshold, include_tests: bool) -> Report:
    """Run the full analysis pipeline: discover → parse → graph → heuristics → score."""
    from .discovery import discover_workspace, enumerate_python_files
    from .graph import ImportGraph
    from .heuristics import branch_density
    from .heuristics import context_bombs
    from .heuristics import coupling
    from .heuristics import dead_weight
    from .heuristics import duplication
    from .heuristics import import_diversity
    from .heuristics import reexport_entropy
    from .heuristics import error_swallow
    from .heuristics import unused_imports
    from .heuristics import deep_nesting
    from .heuristics import dangerous_pattern
    from .heuristics import naming_entropy
    from .heuristics import type_complexity
    from .heuristics import implicit_control
    from .parsing import FileInfo, extract_file_info
    from .scoring import compute_global_score

    workspace = discover_workspace(path)
    pfiles = enumerate_python_files(workspace.packages, workspace.root, include_tests)

    # Parse every file (sequential; parallel via ProcessPoolExecutor could be added)
    infos: list[FileInfo] = []
    files_skipped = 0
    total_lines = 0
    total_estimated_tokens = 0
    errors: list[str] = []

    for pf in pfiles:
        info = extract_file_info(pf.path, pf.package_name, workspace.root)
        if info is None:
            files_skipped += 1
            errors.append(f"Parse failed: {pf.relative_path}")
            continue
        infos.append(info)
        total_lines += pf.line_count
        total_estimated_tokens += context_bombs.estimate_tokens(pf.size_bytes)

    graph = ImportGraph.build(infos, workspace.root)

    # Build path->info lookup map to avoid O(n^2) scans per file
    info_by_path: dict[Path, FileInfo] = {info.path: info for info in infos}

    # Run all heuristics
    findings: list[Finding] = []
    counter = 0

    for pf in pfiles:
        info = info_by_path.get(pf.path)
        if info is None:
            continue

        # Per-file heuristics (with error isolation matching run-heuristics.ts)
        try:
            result = context_bombs.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: context-bombs: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = reexport_entropy.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: reexport-entropy: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = coupling.analyze(pf, graph, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: coupling: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = unused_imports.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: unused-imports: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = error_swallow.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: error-swallow: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = branch_density.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: branch-density: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = deep_nesting.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: deep-nesting: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = import_diversity.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: import-diversity: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = dangerous_pattern.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: dangerous-pattern: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = naming_entropy.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: naming-entropy: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = type_complexity.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: type-complexity: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1

        try:
            result = implicit_control.analyze(pf, info, threshold, counter + 1)
        except Exception as exc:
            errors.append(f"{pf.relative_path}: implicit-control: {exc}")
        else:
            if result:
                findings.append(result)
                counter += 1


    # Global heuristics (operate on all files at once)
    try:
        dw_findings = dead_weight.analyze_orphaned_files(pfiles, infos, graph, counter)
    except Exception as exc:
        errors.append(f"dead-weight: {exc}")
    else:
        findings.extend(dw_findings)
        counter += len(dw_findings)

    try:
        dup_findings = duplication.analyze_duplicates(infos, counter)
    except Exception as exc:
        errors.append(f"duplication: {exc}")
    else:
        findings.extend(dup_findings)
        counter += len(dup_findings)

    # Sort by severity weight descending
    findings.sort(key=lambda f: f.severity.weight, reverse=True)

    file_count = len(infos) + files_skipped
    global_score = compute_global_score(findings, file_count, total_estimated_tokens)

    return Report(
        findings=findings,
        global_score=global_score,
        files_analyzed=len(infos),
        files_skipped=files_skipped,
        total_lines=total_lines,
        total_estimated_tokens=total_estimated_tokens,
        errors=errors,
        version=__version__,
    )


def render_report(report: Report, fmt: str, no_color: bool = False) -> str:
    """Dispatch to the appropriate reporter."""
    if fmt == "json":
        from .reporters.json_report import render_json

        return render_json(report)
    elif fmt == "md":
        from .reporters.markdown import render_markdown

        return render_markdown(report)
    elif fmt == "llm":
        from .reporters.llm import render_llm

        return render_llm(report)
    else:
        from .reporters.terminal import render_terminal

        return render_terminal(report, no_color=no_color)


def main(argv: list[str] | None = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)

    logging.basicConfig(
        level=logging.WARNING, format="%(levelname)s: %(message)s", stream=sys.stderr
    )

    threshold = Threshold(args.threshold)
    include_tests = getattr(args, "include_tests", False)

    if args.command == "explain":
        _do_explain(args.path, args.root)
        return

    path: Path = args.path
    fmt: str = getattr(args, "format", "json" if args.command == "json" else "text")
    no_color: bool = getattr(args, "no_color", False)

    report = analyze(path, threshold, include_tests)
    output = render_report(report, fmt, no_color=no_color)

    try:
        sys.stdout.write(output)
    except BrokenPipeError:
        pass


def _do_explain(file_path: Path, root: Path) -> None:
    """Stub — explain command to be implemented."""
    print(f"explain: not yet implemented (file={file_path}, root={root})")
