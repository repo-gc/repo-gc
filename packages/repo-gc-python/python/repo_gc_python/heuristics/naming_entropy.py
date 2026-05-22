"""Naming entropy / mixed-style heuristic.

Detects files that use multiple naming conventions (snake_case, camelCase,
PascalCase, SCREAMING_SNAKE) in significant proportion. Mixed naming
conventions within a file can confuse LLM expectations and reduce
the model's ability to reason about the code.
"""

import json

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity


def _classify_identifier(name: str) -> str:
    """Classify an identifier string into a naming convention."""
    if len(name) < 3:
        return "other"

    has_underscore = "_" in name
    all_upper = name == name.upper()
    all_lower = name == name.lower()
    first_upper = name[0].isupper()
    first_lower = name[0].islower()

    if has_underscore and all_lower:
        return "snake_case"
    if has_underscore and all_upper:
        return "SCREAMING_SNAKE"
    if first_lower and any(c.isupper() for c in name):
        return "camelCase"
    if first_upper and any(c.islower() for c in name[1:]):
        return "PascalCase"
    return "other"


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    identifiers = info.all_identifiers

    # Require a minimum number of identifiers to classify
    if len(identifiers) < 10:
        return None

    # Classify each identifier
    counts: dict[str, int] = {}
    total = len(identifiers)

    for ident in identifiers:
        convention = _classify_identifier(ident)
        counts[convention] = counts.get(convention, 0) + 1

    # Find conventions that represent >5% of total
    threshold_count = total * 0.05
    active_conventions = [
        conv for conv, count in counts.items() if count > threshold_count
    ]

    # If 3+ conventions are in significant use, emit a finding
    if len(active_conventions) < 3:
        return None

    # Find the dominant convention
    dominant_convention = max(counts, key=counts.get)

    return Finding(
        id=f"ne-{counter:03d}",
        kind=FindingKind.NamingEntropy,
        severity=Severity.Low,
        confidence=0.50,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"convention_counts: {json.dumps(counts)}",
            f"dominant_convention: {dominant_convention}",
            f"mixed_count: {len(active_conventions)}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
