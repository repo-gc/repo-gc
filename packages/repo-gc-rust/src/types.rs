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
    pub fn llm_label(&self) -> &'static str {
        match self {
            Severity::Critical => "C",
            Severity::High => "H",
            Severity::Medium => "M",
            Severity::Low => "L",
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
    BranchDensity,
    DeepNesting,
    TypeComplexity,
    CommentRatio,
    ImplicitControl,
    ErrorSwallow,
    DangerousPattern,
    NamingEntropy,
    StringlyTyped,
    ImportDiversity,
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
            FindingKind::BranchDensity => "branch-density",
            FindingKind::DeepNesting => "deep-nesting",
            FindingKind::TypeComplexity => "type-complexity",
            FindingKind::CommentRatio => "comment-ratio",
            FindingKind::ImplicitControl => "implicit-control",
            FindingKind::ErrorSwallow => "error-swallow",
            FindingKind::DangerousPattern => "dangerous-pattern",
            FindingKind::NamingEntropy => "naming-entropy",
            FindingKind::StringlyTyped => "stringly-typed",
            FindingKind::ImportDiversity => "import-diversity",
        }
    }
    /// Short code for LLM/token-efficient output
    pub fn llm_label(&self) -> &'static str {
        match self {
            FindingKind::ContextBomb => "OVS",
            FindingKind::DeadWeight => "DEAD",
            FindingKind::ReexportEntropy => "EXP",
            FindingKind::CouplingHotspot => "COUP",
            FindingKind::CodeDuplication => "DUP",
            FindingKind::UnusedImport => "ZOMB",
            FindingKind::BranchDensity => "BRAN",
            FindingKind::DeepNesting => "NEST",
            FindingKind::TypeComplexity => "TYPE",
            FindingKind::CommentRatio => "CMNT",
            FindingKind::ImplicitControl => "HIDE",
            FindingKind::ErrorSwallow => "SWAL",
            FindingKind::DangerousPattern => "DANG",
            FindingKind::NamingEntropy => "MIXD",
            FindingKind::StringlyTyped => "STRY",
            FindingKind::ImportDiversity => "GODF",
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
    pub ai_friction_score: u32,
    pub context_waste_score: u32,
    pub structural_entropy_score: u32,
    pub reasoning_complexity_score: u32,
    pub context_waste_ratio: f64,
    pub estimated_waste_pct: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub global_score: GlobalScore,
    pub files_analyzed: usize,
    pub files_skipped: usize,
    pub total_lines: usize,
    pub total_estimated_tokens: usize,
    pub errors: Vec<String>,
    pub version: String,
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
        assert_eq!(FindingKind::BranchDensity.label(), "branch-density");
        assert_eq!(FindingKind::DeepNesting.label(), "deep-nesting");
        assert_eq!(FindingKind::TypeComplexity.label(), "type-complexity");
        assert_eq!(FindingKind::CommentRatio.label(), "comment-ratio");
        assert_eq!(FindingKind::ImplicitControl.label(), "implicit-control");
        assert_eq!(FindingKind::ErrorSwallow.label(), "error-swallow");
        assert_eq!(FindingKind::DangerousPattern.label(), "dangerous-pattern");
        assert_eq!(FindingKind::NamingEntropy.label(), "naming-entropy");
        assert_eq!(FindingKind::StringlyTyped.label(), "stringly-typed");
        assert_eq!(FindingKind::ImportDiversity.label(), "import-diversity");
    }

    #[test]
    fn finding_kind_llm_labels_are_stable() {
        assert_eq!(FindingKind::ContextBomb.llm_label(), "OVS");
        assert_eq!(FindingKind::BranchDensity.llm_label(), "BRAN");
        assert_eq!(FindingKind::DeepNesting.llm_label(), "NEST");
        assert_eq!(FindingKind::TypeComplexity.llm_label(), "TYPE");
        assert_eq!(FindingKind::CommentRatio.llm_label(), "CMNT");
        assert_eq!(FindingKind::ImplicitControl.llm_label(), "HIDE");
        assert_eq!(FindingKind::ErrorSwallow.llm_label(), "SWAL");
        assert_eq!(FindingKind::DangerousPattern.llm_label(), "DANG");
        assert_eq!(FindingKind::NamingEntropy.llm_label(), "MIXD");
        assert_eq!(FindingKind::StringlyTyped.llm_label(), "STRY");
        assert_eq!(FindingKind::ImportDiversity.llm_label(), "GODF");
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
