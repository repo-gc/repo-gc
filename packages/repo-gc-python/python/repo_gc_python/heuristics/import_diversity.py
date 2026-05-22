"""Import diversity (GODF) heuristic.

Detects files that import from too many unrelated domains — a "god file"
that does too much. Backed by the Program Decomposition paper
(arXiv 2401.12412): reducing cross-file dependencies shrinks context to
~5% of window.

Logic: count distinct first-segment domains across all imported modules.
If count > limit -> finding.
Severity: >= limit*3 -> High, >= limit*2 -> Medium, >= limit -> Low.
"""

from ..cli import Threshold
from ..discovery import PythonFile
from ..parsing import FileInfo
from ..types import Finding, FindingKind, Severity


def _extract_domain(module_path: str) -> str | None:
    """Extract the top-level domain from a Python module path.

    - "os.path.join" -> "os"
    - "numpy.linalg" -> "numpy"
    - "mypackage.utils.helpers" -> "mypackage"
    - ".module" or "..parent.module" -> None (relative, skipped)
    """
    if module_path.startswith("."):
        return None  # relative import
    parts = module_path.split(".")
    return parts[0]


def analyze(
    pfile: PythonFile,
    info: FileInfo,
    threshold: Threshold,
    counter: int,
) -> Finding | None:
    limit = threshold.import_domain_limit

    domains: set[str] = set()
    for mod_path in info.imported_modules:
        domain = _extract_domain(mod_path)
        if domain:
            domains.add(domain)

    count = len(domains)

    if count <= limit:
        return None

    if count >= limit * 3:
        severity = Severity.High
    elif count >= limit * 2:
        severity = Severity.Medium
    else:
        severity = Severity.Low

    # Top 8 domains (sorted alphabetically)
    top_domains = ", ".join(sorted(domains)[:8])

    return Finding(
        id=f"id-{counter:03d}",
        kind=FindingKind.ImportDiversity,
        severity=severity,
        confidence=0.70,
        path=str(pfile.relative_path),
        summary="",
        reasons=[],
        evidence=[
            f"domain_count: {count}",
            f"domains: {top_domains}",
            f"limit: {limit}",
        ],
        suggested_next_step="",
        estimated_tokens=None,
    )
