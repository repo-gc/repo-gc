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
    reasons = [f"{pfile.line_count} lines (limit: {limit})"]
    if info.function_count > 10:
        reasons.append(f"{info.function_count} functions defined")
    if info.class_count > 3:
        reasons.append(f"{info.class_count} classes defined")

    return Finding(
        id=f"cb-{counter:03d}",
        kind=FindingKind.ContextBomb,
        severity=severity,
        confidence=0.95 if pfile.line_count >= limit * 2 else 0.75,
        path=str(pfile.relative_path),
        summary=(
            f"Oversized file — {pfile.line_count} lines / "
            f"~{estimated_tokens_} tokens, each AI edit re-reads this entire file"
        ),
        reasons=reasons,
        evidence=[
            f"line_count: {pfile.line_count}",
            f"estimated_tokens: {estimated_tokens_}",
        ],
        suggested_next_step=(
            f"Split {pfile.relative_path} into smaller modules "
            f"(target <{limit} lines each)"
        ),
        estimated_tokens=estimated_tokens_,
    )
