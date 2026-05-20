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
    from .heuristics import context_bombs
    from .heuristics import coupling
    from .heuristics import dead_weight
    from .heuristics import duplication
    from .heuristics import reexport_entropy
    from .heuristics import unused_imports
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

    # Run all heuristics
    findings: list[Finding] = []
    counter = 0

    for i, pf in enumerate(pfiles):
        info = next((x for x in infos if x.path == pf.path), None)
        if info is None:
            continue

        # Per-file heuristics
        result = context_bombs.analyze(pf, info, threshold, counter + 1)
        if result:
            findings.append(result)
            counter += 1

        result = reexport_entropy.analyze(pf, info, threshold, counter + 1)
        if result:
            findings.append(result)
            counter += 1

        result = coupling.analyze(pf, graph, threshold, counter + 1)
        if result:
            findings.append(result)
            counter += 1

        result = unused_imports.analyze(pf, info, threshold, counter + 1)
        if result:
            findings.append(result)
            counter += 1

    # Global heuristics (operate on all files at once)
    dw_findings = dead_weight.analyze_orphaned_files(pfiles, infos, counter)
    findings.extend(dw_findings)
    counter += len(dw_findings)

    dup_findings = duplication.analyze_duplicates(infos, counter)
    findings.extend(dup_findings)

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
