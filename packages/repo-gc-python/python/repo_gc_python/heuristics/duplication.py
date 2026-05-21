"""Duplicate function body detection — mirrors Rust's duplication.rs."""

from __future__ import annotations

import ast
from collections import defaultdict

from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity

MIN_BODY_LEN = 40

# Python keywords — used to exclude them from identifier normalization.
_KEYWORDS: frozenset[str] = frozenset({
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
    "try", "while", "with", "yield", "match", "case", "type",
})


def _normalize(body: str) -> str:
    """Strip all whitespace from a body string for stable comparison."""
    return "".join(body.split())


def _is_trivial_body(raw_body: str) -> bool:
    """Return True if *raw_body* is a single-statement delegation.

    These structural template functions (e.g. ``encode_impl(...)`` wrappers)
    are not meaningful duplication targets.
    """
    try:
        tree = ast.parse(raw_body)
    except SyntaxError:
        return False
    # Count statements in the module body
    stmts = [s for s in ast.walk(tree) if isinstance(s, ast.stmt)]
    # Module itself counts as one, so ≤ 1 more means empty or single-statement
    return len(stmts) <= 1


def _is_test_only_name(name: str) -> bool:
    """Skip functions named like test-only entry points."""
    return "for_test" in name


class _IdentNormalizer(ast.NodeTransformer):
    """Replace all Name nodes with ``_`` for Type 2 clone detection."""

    def visit_Name(self, node: ast.Name) -> ast.Name:
        if node.id not in _KEYWORDS:
            return ast.Name(id="_", ctx=node.ctx)
        return node


def _normalize_identifiers(raw_body: str) -> str:
    """Replace identifiers with ``_`` so renamed-variable clones match."""
    try:
        tree = ast.parse(raw_body)
    except SyntaxError:
        return _normalize(raw_body)
    normalizer = _IdentNormalizer()
    normalized = normalizer.visit(tree)
    return _normalize(ast.unparse(normalized))


def analyze_duplicates(infos: list[FileInfo], counter: int) -> list[Finding]:
    body_map: dict[str, list[tuple[str, str]]] = defaultdict(list)

    for info in infos:
        for fn_name, raw_body, norm_body in info.function_bodies:
            if _is_test_only_name(fn_name):
                continue
            # Check raw body length before identifier normalization,
            # since normalization collapses all idents to `_`.
            if len(norm_body) < MIN_BODY_LEN:
                continue
            if _is_trivial_body(raw_body):
                continue
            key = _normalize_identifiers(raw_body)
            body_map[key].append((str(info.path), fn_name))

    findings: list[Finding] = []

    for _body, locations in body_map.items():
        unique_files = {path for path, _ in locations}
        if len(unique_files) < 2:
            continue

        file_count = len(unique_files)
        if file_count >= 5:
            severity = Severity.High
        elif file_count >= 3:
            severity = Severity.Medium
        else:
            severity = Severity.Low

        counter += 1
        files_list = [path for path, _ in locations[:5]]

        findings.append(
            Finding(
                id=f"dup-{counter:03d}",
                kind=FindingKind.CodeDuplication,
                severity=severity,
                confidence=0.85,
                path=locations[0][0],
                summary="",
                reasons=[],
                evidence=[
                    f"file_count: {file_count}",
                    f"fn_name: {locations[0][1]}",
                    f"files: {', '.join(files_list)}",
                ],
                suggested_next_step="",
                estimated_tokens=None,
            )
        )

    findings.sort(key=lambda f: f.severity.weight, reverse=True)
    return findings
