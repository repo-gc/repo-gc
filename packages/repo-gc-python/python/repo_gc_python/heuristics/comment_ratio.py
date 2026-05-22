"""Comment ratio heuristic.

Detects files with too few comments (LLMs can't infer intent) or too many
(token waste). Uses research-backed thresholds: min 0.03 (3%), max 0.20 (20%)
for strict mode.
"""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    line_count = max(pfile.line_count, 1)
    ratio = info.comment_line_count / line_count

    if ratio < threshold.comment_ratio_min:
        return Finding(
            id=f"cr-{counter:03d}",
            kind=FindingKind.CommentRatio,
            severity=Severity.Medium,
            confidence=0.65,
            path=str(pfile.relative_path),
            summary="",
            reasons=[],
            evidence=[
                f"comment_lines: {info.comment_line_count}",
                f"total_lines: {pfile.line_count}",
                f"ratio: {ratio:.4f}",
                "direction: sparse",
            ],
            suggested_next_step="",
            estimated_tokens=None,
        )

    if ratio > threshold.comment_ratio_max:
        return Finding(
            id=f"cr-{counter:03d}",
            kind=FindingKind.CommentRatio,
            severity=Severity.Low,
            confidence=0.65,
            path=str(pfile.relative_path),
            summary="",
            reasons=[],
            evidence=[
                f"comment_lines: {info.comment_line_count}",
                f"total_lines: {pfile.line_count}",
                f"ratio: {ratio:.4f}",
                "direction: verbose",
            ],
            suggested_next_step="",
            estimated_tokens=None,
        )

    return None
