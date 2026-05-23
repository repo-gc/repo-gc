"""Shared types mirroring the Rust types.rs schema.

Finding kinds serialize to PascalCase (e.g. "ContextBomb") to match the
serde-derived JSON output consumed by the TypeScript runner's normalizeFindings.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field, asdict
from enum import Enum
from pathlib import Path
from typing import Optional


_CANONICAL: dict | None = None


def _load_canonical() -> dict:
    """Load the canonical finding kinds JSON (single source of truth).

    The file lives at the repo root under test-fixtures/finding-kinds.json.
    """
    global _CANONICAL
    if _CANONICAL is None:
        path = (
            Path(__file__).resolve().parent.parent.parent.parent.parent
            / "test-fixtures"
            / "finding-kinds.json"
        )
        with open(path) as f:
            _CANONICAL = json.load(f)
    return _CANONICAL


def _severity_data(severity_id: str) -> dict:
    """Look up a severity entry by its PascalCase id from the canonical JSON."""
    for s in _load_canonical()["severities"]:
        if s["id"] == severity_id:
            return s
    return {}


def _kind_data(kind_id: str) -> dict:
    """Look up a finding kind entry by its PascalCase id from the canonical JSON."""
    for k in _load_canonical()["finding_kinds"]:
        if k["id"] == kind_id:
            return k
    return {}


class Severity(Enum):
    Critical = "Critical"
    High = "High"
    Medium = "Medium"
    Low = "Low"

    @property
    def weight(self) -> float:
        return _severity_data(self.value).get("weight", 1.0)

    @property
    def label(self) -> str:
        return _severity_data(self.value).get("label", self.value.upper())

    @property
    def llm_label(self) -> str:
        return _severity_data(self.value).get("llm_label", self.value[0])


class FindingKind(Enum):
    ContextBomb = "ContextBomb"
    DeadWeight = "DeadWeight"
    ReexportEntropy = "ReexportEntropy"
    CouplingHotspot = "CouplingHotspot"
    CodeDuplication = "CodeDuplication"
    UnusedImport = "UnusedImport"
    BranchDensity = "BranchDensity"
    DeepNesting = "DeepNesting"
    TypeComplexity = "TypeComplexity"
    CommentRatio = "CommentRatio"
    ImplicitControl = "ImplicitControl"
    ErrorSwallow = "ErrorSwallow"
    DangerousPattern = "DangerousPattern"
    NamingEntropy = "NamingEntropy"
    StringlyTyped = "StringlyTyped"
    ImportDiversity = "ImportDiversity"

    @property
    def label(self) -> str:
        return _kind_data(self.value).get("label", self.value.lower())

    @property
    def llm_label(self) -> str:
        return _kind_data(self.value).get("llm_code", "")

    @property
    def category(self) -> str:
        """Return the scoring category (e.g. 'context_waste', 'structural_entropy', 'reasoning_complexity')."""
        return _kind_data(self.value).get("category", "")


@dataclass
class Finding:
    id: str
    kind: FindingKind
    severity: Severity
    confidence: float
    path: str
    summary: str
    reasons: list[str] = field(default_factory=list)
    evidence: list[str] = field(default_factory=list)
    suggested_next_step: str = ""
    estimated_tokens: Optional[int] = None


@dataclass
class GlobalScore:
    ai_friction_score: int
    context_waste_score: int
    structural_entropy_score: int
    reasoning_complexity_score: int
    context_waste_ratio: float
    estimated_waste_pct: int


@dataclass
class Report:
    findings: list[Finding]
    global_score: GlobalScore
    files_analyzed: int = 0
    files_skipped: int = 0
    total_lines: int = 0
    total_estimated_tokens: int = 0
    errors: list[str] = field(default_factory=list)
    version: str = ""


class ReportEncoder(json.JSONEncoder):
    """Custom encoder that matches Rust's serde JSON output.

    Serializes FindingKind and Severity as their PascalCase enum name
    (e.g. "ContextBomb", "Critical") rather than the value string.
    Path objects become plain strings.
    Finding/GlobalScore/Report use dataclass asdict.
    """

    def default(self, obj):
        if isinstance(obj, (Severity, FindingKind)):
            return obj.name
        if isinstance(obj, Path):
            return str(obj)
        if isinstance(obj, (Report, GlobalScore, Finding)):
            return asdict(obj)
        return super().default(obj)


def report_to_json(report: Report) -> str:
    """Serialize a Report to JSON matching the Rust serde output schema."""
    return json.dumps(report, cls=ReportEncoder, indent=2)
