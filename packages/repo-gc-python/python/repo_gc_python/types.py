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


class Severity(Enum):
    Critical = "Critical"
    High = "High"
    Medium = "Medium"
    Low = "Low"

    @property
    def weight(self) -> float:
        return {Severity.Critical: 4.0, Severity.High: 2.0, Severity.Medium: 1.0, Severity.Low: 0.5}[self]

    @property
    def label(self) -> str:
        return {Severity.Critical: "CRITICAL", Severity.High: "HIGH", Severity.Medium: "MEDIUM", Severity.Low: "LOW"}[self]

    @property
    def llm_label(self) -> str:
        return {Severity.Critical: "C", Severity.High: "H", Severity.Medium: "M", Severity.Low: "L"}[self]


class FindingKind(Enum):
    ContextBomb = "ContextBomb"
    DeadWeight = "DeadWeight"
    ReexportEntropy = "ReexportEntropy"
    CouplingHotspot = "CouplingHotspot"
    CodeDuplication = "CodeDuplication"
    UnusedImport = "UnusedImport"

    @property
    def label(self) -> str:
        _map = {
            FindingKind.ContextBomb: "context-bomb",
            FindingKind.DeadWeight: "dead-weight",
            FindingKind.ReexportEntropy: "reexport-entropy",
            FindingKind.CouplingHotspot: "coupling-hotspot",
            FindingKind.CodeDuplication: "code-duplication",
            FindingKind.UnusedImport: "unused-import",
        }
        return _map[self]

    @property
    def llm_label(self) -> str:
        _map = {
            FindingKind.ContextBomb: "OVS",
            FindingKind.DeadWeight: "DEAD",
            FindingKind.ReexportEntropy: "EXP",
            FindingKind.CouplingHotspot: "COUP",
            FindingKind.CodeDuplication: "DUP",
            FindingKind.UnusedImport: "ZOMB",
        }
        return _map[self]


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
