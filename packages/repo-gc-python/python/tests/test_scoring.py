"""Test scoring — direct port of Rust scoring.rs tests."""

from repo_gc_python.scoring import compute_global_score
from repo_gc_python.types import Finding, FindingKind, Severity


def mkf(kind: FindingKind, sev: Severity) -> Finding:
    return Finding(
        id="x",
        kind=kind,
        severity=sev,
        confidence=1.0,
        path="src/lib.py",
        summary="",
    )


def test_zero_findings_zero_score():
    s = compute_global_score([], 10, 0)
    assert s.ai_friction_score == 0
    assert s.context_waste_score == 0


def test_score_capped_at_100():
    ff = [mkf(FindingKind.ContextBomb, Severity.Critical) for _ in range(100)]
    assert compute_global_score(ff, 5, 0).ai_friction_score <= 100


def test_more_severe_means_higher_score():
    low = [mkf(FindingKind.ContextBomb, Severity.Low)]
    high = [mkf(FindingKind.ContextBomb, Severity.Critical)]
    assert (
        compute_global_score(high, 10, 0).ai_friction_score
        > compute_global_score(low, 10, 0).ai_friction_score
    )
