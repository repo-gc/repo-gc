use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn weight(&self) -> f32 {
        match self {
            Severity::Critical => 4.0,
            Severity::High => 2.0,
            Severity::Medium => 1.0,
            Severity::Low => 0.5,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FindingKind {
    ContextBomb,
    DeadWeight,
    ReexportEntropy,
    CouplingHotspot,
    CodeDuplication,
    UnusedImport,
}

impl FindingKind {
    pub fn label(&self) -> &'static str {
        match self {
            FindingKind::ContextBomb => "context-bomb",
            FindingKind::DeadWeight => "dead-weight",
            FindingKind::ReexportEntropy => "reexport-entropy",
            FindingKind::CouplingHotspot => "coupling-hotspot",
            FindingKind::CodeDuplication => "code-duplication",
            FindingKind::UnusedImport => "unused-import",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub kind: FindingKind,
    pub severity: Severity,
    pub confidence: f32,
    pub path: PathBuf,
    pub summary: String,
    pub reasons: Vec<String>,
    pub evidence: Vec<String>,
    pub suggested_next_step: String,
    pub estimated_tokens: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalScore {
    pub ai_hostility_score: u32,
    pub context_waste_score: u32,
    pub entropy_score: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub global_score: GlobalScore,
    pub files_analyzed: usize,
    pub files_skipped: usize,
    pub total_lines: usize,
    pub total_estimated_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_weights_are_ordered() {
        assert!(Severity::Critical.weight() > Severity::High.weight());
        assert!(Severity::High.weight() > Severity::Medium.weight());
        assert!(Severity::Medium.weight() > Severity::Low.weight());
    }

    #[test]
    fn finding_kind_labels_are_stable() {
        assert_eq!(FindingKind::ContextBomb.label(), "context-bomb");
        assert_eq!(FindingKind::CouplingHotspot.label(), "coupling-hotspot");
    }

    #[test]
    fn finding_round_trips_json() {
        let f = Finding {
            id: "cb-001".into(),
            kind: FindingKind::ContextBomb,
            severity: Severity::High,
            confidence: 0.9,
            path: PathBuf::from("src/lib.rs"),
            summary: "big".into(),
            reasons: vec![],
            evidence: vec![],
            suggested_next_step: "split".into(),
            estimated_tokens: Some(12000),
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "cb-001");
    }
}
