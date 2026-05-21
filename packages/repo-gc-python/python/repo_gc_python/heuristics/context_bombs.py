"""Oversized file detection — mirrors Rust's context_bombs.rs."""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity


def estimate_tokens(size_bytes: int) -> int:
    """4 bytes ≈ 1 token (LLM tokenizer heuristic)."""
    return size_bytes // 4


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    limit = threshold.line_count_limit
    if pfile.line_count < limit:
        return None

    if pfile.line_count >= limit * 4:
        severity = Severity.Critical
    elif pfile.line_count >= limit * 2:
        severity = Severity.High
    else:
        severity = Severity.Medium

    estimated_tokens_ = estimate_tokens(pfile.size_bytes)

    evidence = [
        f"line_count: {pfile.line_count}",
        f"estimated_tokens: {estimated_tokens_}",
        f"limit: {limit}",
    ]
    if info.function_count > 10:
        evidence.append(f"function_count: {info.function_count}")
    if info.class_count > 3:
        evidence.append(f"class_count: {info.class_count}")

    return Finding(
        id=f"cb-{counter:03d}",
        kind=FindingKind.ContextBomb,
        severity=severity,
        confidence=0.95 if pfile.line_count >= limit * 4 else 0.85 if pfile.line_count >= limit * 2 else 0.75,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=evidence,
        suggested_next_step="",
        estimated_tokens=estimated_tokens_,
    )
