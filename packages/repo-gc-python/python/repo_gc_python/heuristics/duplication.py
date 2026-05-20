"""Duplicate function body detection — mirrors Rust's duplication.rs."""

from collections import defaultdict

from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity

MIN_BODY_LEN = 40


def _normalize(body: str) -> str:
    """Strip all whitespace from a body string for stable comparison."""
    return "".join(body.split())


def analyze_duplicates(infos: list[FileInfo], counter: int) -> list[Finding]:
    body_map: dict[str, list[tuple[str, str]]] = defaultdict(list)

    for info in infos:
        for fn_name, body in info.function_bodies:
            key = _normalize(body)
            if len(key) < MIN_BODY_LEN:
                continue
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
        evidence = [f"{path} :: {name}" for path, name in locations[:4]]

        findings.append(
            Finding(
                id=f"dup-{counter:03d}",
                kind=FindingKind.CodeDuplication,
                severity=severity,
                confidence=0.85,
                path=locations[0][0],
                summary=(
                    f"Duplicate logic — fn `{locations[0][1]}` copied across "
                    f"{file_count} files, AI edits will not propagate"
                ),
                reasons=[
                    f"Identical function body found in {file_count} different files — "
                    f"AI edits will not propagate"
                ],
                evidence=evidence,
                suggested_next_step=(
                    "DRY it up: extract duplicated logic into a shared utility function"
                ),
                estimated_tokens=None,
            )
        )

    findings.sort(key=lambda f: f.severity.weight, reverse=True)
    return findings
