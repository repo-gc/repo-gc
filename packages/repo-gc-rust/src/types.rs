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
        crate::canonical::severity_weight(&format!("{:?}", self)) as f32
    }
    pub fn label(&self) -> &'static str {
        crate::canonical::severity_label(&format!("{:?}", self))
    }
    pub fn llm_label(&self) -> &'static str {
        crate::canonical::severity_llm_label(&format!("{:?}", self))
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Critical" => Ok(Severity::Critical),
            "High" => Ok(Severity::High),
            "Medium" => Ok(Severity::Medium),
            "Low" => Ok(Severity::Low),
            _ => Err(format!("unknown Severity variant: {s}")),
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
        crate::canonical::kind_label(&format!("{:?}", self))
    }
    /// Short code for LLM/token-efficient output
    pub fn llm_label(&self) -> &'static str {
        crate::canonical::kind_llm_code(&format!("{:?}", self))
    }
}

impl std::str::FromStr for FindingKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ContextBomb" => Ok(FindingKind::ContextBomb),
            "DeadWeight" => Ok(FindingKind::DeadWeight),
            "ReexportEntropy" => Ok(FindingKind::ReexportEntropy),
            "CouplingHotspot" => Ok(FindingKind::CouplingHotspot),
            "CodeDuplication" => Ok(FindingKind::CodeDuplication),
            "UnusedImport" => Ok(FindingKind::UnusedImport),
            "BranchDensity" => Ok(FindingKind::BranchDensity),
            "DeepNesting" => Ok(FindingKind::DeepNesting),
            "TypeComplexity" => Ok(FindingKind::TypeComplexity),
            "CommentRatio" => Ok(FindingKind::CommentRatio),
            "ImplicitControl" => Ok(FindingKind::ImplicitControl),
            "ErrorSwallow" => Ok(FindingKind::ErrorSwallow),
            "DangerousPattern" => Ok(FindingKind::DangerousPattern),
            "NamingEntropy" => Ok(FindingKind::NamingEntropy),
            "StringlyTyped" => Ok(FindingKind::StringlyTyped),
            "ImportDiversity" => Ok(FindingKind::ImportDiversity),
            _ => Err(format!("unknown FindingKind variant: {s}")),
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

impl Finding {
    pub fn new(
        id: String,
        kind: FindingKind,
        severity: Severity,
        confidence: f32,
        path: PathBuf,
        evidence: Vec<String>,
    ) -> Self {
        Finding {
            id,
            kind,
            severity,
            confidence,
            path,
            summary: String::new(),
            reasons: vec![],
            suggested_next_step: String::new(),
            estimated_tokens: None,
            evidence,
        }
    }
}

/// Returns the next finding ID with the given prefix, incrementing the counter.
/// Every heuristic uses `XX-{:03}` format, so this is the single source of truth.
pub fn next_finding_id(prefix: &str, counter: &mut usize) -> String {
    *counter += 1;
    format!("{}-{:03}", prefix, counter)
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

impl Default for GlobalScore {
    fn default() -> Self {
        GlobalScore {
            ai_friction_score: 0,
            context_waste_score: 0,
            structural_entropy_score: 0,
            reasoning_complexity_score: 0,
            context_waste_ratio: 0.0,
            estimated_waste_pct: 0,
        }
    }
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
    fn canonical_json_matches_enum_variants() {
        for &id in crate::canonical::kind_ids() {
            assert!(
                id.parse::<FindingKind>().is_ok(),
                "unknown FindingKind variant: {id}",
            );
        }
        for &id in crate::canonical::severity_ids() {
            assert!(
                id.parse::<Severity>().is_ok(),
                "unknown Severity variant: {id}",
            );
        }
    }

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
