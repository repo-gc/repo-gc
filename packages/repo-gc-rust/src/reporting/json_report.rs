use anyhow::Result;
use crate::types::Report;

pub fn render(report: &Report) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Finding, FindingKind, GlobalScore, Severity};
    use std::path::PathBuf;

    #[test]
    fn renders_valid_json_with_files_skipped() {
        let report = Report {
            findings: vec![Finding {
                id: "cb-001".into(),
                kind: FindingKind::ContextBomb,
                severity: Severity::High,
                confidence: 0.9,
                path: PathBuf::from("src/lib.rs"),
                summary: "Large".into(),
                reasons: vec![],
                evidence: vec![],
                suggested_next_step: "Split".into(),
                estimated_tokens: Some(10000),
            }],
            global_score: GlobalScore {
                ai_friction_score: 55,
                context_waste_score: 40,
                structural_entropy_score: 20,
                context_waste_ratio: 0.39,
                estimated_waste_pct: 8,
            reasoning_complexity_score: 0,
            },
            files_analyzed: 10,
            files_skipped: 2,
            total_lines: 5000,
            total_estimated_tokens: 50000,
            errors: vec![],
            version: "0.0.0".to_string(),
        };
        let v: serde_json::Value = serde_json::from_str(&render(&report).unwrap()).unwrap();
        assert_eq!(v["global_score"]["ai_friction_score"], 55);
        assert_eq!(v["files_skipped"], 2);
        assert!(v["findings"].is_array());
    }
}
