"""Test type serialization — ensures JSON output matches Rust serde format."""

import json

from repo_gc_python.types import (
    Finding,
    FindingKind,
    GlobalScore,
    Report,
    ReportEncoder,
    Severity,
    report_to_json,
)


def test_severity_weights_are_ordered():
    assert Severity.Critical.weight > Severity.High.weight
    assert Severity.High.weight > Severity.Medium.weight
    assert Severity.Medium.weight > Severity.Low.weight


def test_finding_kind_labels_are_stable():
    assert FindingKind.ContextBomb.label == "context-bomb"
    assert FindingKind.CouplingHotspot.label == "coupling-hotspot"


def test_severity_serializes_pascal_case():
    f = Finding(
        id="cb-001",
        kind=FindingKind.ContextBomb,
        severity=Severity.High,
        confidence=0.9,
        path="src/lib.py",
        summary="big",
    )
    j = json.dumps(f, cls=ReportEncoder)
    assert '"ContextBomb"' in j
    assert '"High"' in j


def test_finding_round_trips():
    f = Finding(
        id="cb-001",
        kind=FindingKind.ContextBomb,
        severity=Severity.High,
        confidence=0.9,
        path="src/lib.py",
        summary="big",
        reasons=["reason1"],
        evidence=["ev1"],
        suggested_next_step="split",
        estimated_tokens=12000,
    )
    j = report_to_json(
        Report(
            findings=[f],
            global_score=GlobalScore(0, 0, 0, 0.0, 0),
            files_analyzed=1,
        )
    )
    parsed = json.loads(j)
    assert parsed["findings"][0]["id"] == "cb-001"
    assert parsed["findings"][0]["kind"] == "ContextBomb"
    assert parsed["findings"][0]["estimated_tokens"] == 12000


def test_empty_report_json():
    r = Report(
        findings=[],
        global_score=GlobalScore(
            ai_friction_score=0,
            context_waste_score=0,
            structural_entropy_score=0,
            context_waste_ratio=0.0,
            estimated_waste_pct=0,
        ),
    )
    j = report_to_json(r)
    parsed = json.loads(j)
    assert parsed["findings"] == []
    assert parsed["files_analyzed"] == 0


def test_global_score_fields():
    j = report_to_json(
        Report(
            findings=[],
            global_score=GlobalScore(72, 40, 30, 1.5, 5),
        )
    )
    parsed = json.loads(j)
    assert parsed["global_score"]["ai_friction_score"] == 72
    assert parsed["global_score"]["context_waste_ratio"] == 1.5
