use crate::types::{Finding, FindingKind, GlobalScore};

pub fn compute_global_score(findings: &[Finding], total_files: usize) -> GlobalScore {
    let max_expected = (total_files as f32 * 1.5).max(1.0);

    let ai_hostility = ((findings
        .iter()
        .map(|f| f.severity.weight() * f.confidence)
        .sum::<f32>()
        / max_expected)
        * 100.0)
        .min(100.0)
        .round() as u32;

    let context_waste = ((findings
        .iter()
        .filter(|f| matches!(f.kind, FindingKind::ContextBomb))
        .map(|f| f.severity.weight())
        .sum::<f32>()
        / max_expected)
        * 100.0)
        .min(100.0)
        .round() as u32;

    let entropy = ((findings
        .iter()
        .filter(|f| {
            matches!(
                f.kind,
                FindingKind::ReexportEntropy | FindingKind::CouplingHotspot | FindingKind::DeadWeight
            )
        })
        .map(|f| f.severity.weight())
        .sum::<f32>()
        / max_expected)
        * 100.0)
        .min(100.0)
        .round() as u32;

    GlobalScore {
        ai_hostility_score: ai_hostility,
        context_waste_score: context_waste,
        entropy_score: entropy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Finding, FindingKind, Severity};
    use std::path::PathBuf;

    fn mkf(kind: FindingKind, sev: Severity) -> Finding {
        Finding {
            id: "x".into(),
            kind,
            severity: sev,
            confidence: 1.0,
            path: PathBuf::from("src/lib.rs"),
            summary: "".into(),
            reasons: vec![],
            evidence: vec![],
            suggested_next_step: "".into(),
            estimated_tokens: None,
        }
    }

    #[test]
    fn zero_findings_zero_score() {
        let s = compute_global_score(&[], 10);
        assert_eq!(s.ai_hostility_score, 0);
        assert_eq!(s.context_waste_score, 0);
    }
    #[test]
    fn score_capped_at_100() {
        let ff: Vec<_> = (0..100)
            .map(|_| mkf(FindingKind::ContextBomb, Severity::Critical))
            .collect();
        assert!(compute_global_score(&ff, 5).ai_hostility_score <= 100);
    }
    #[test]
    fn more_severe_means_higher_score() {
        let low = vec![mkf(FindingKind::ContextBomb, Severity::Low)];
        let high = vec![mkf(FindingKind::ContextBomb, Severity::Critical)];
        assert!(
            compute_global_score(&high, 10).ai_hostility_score
                > compute_global_score(&low, 10).ai_hostility_score
        );
    }
}
