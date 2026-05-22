use crate::types::{Report, Severity};

pub fn render(report: &Report) -> String {
    let mut out = String::from("# repo-gc: AI Context Efficiency Report\n\n");

    let gs = &report.global_score;

    out.push_str(&format!(
        "**Files analyzed:** {}  |  **Total lines:** {}  |  **Est. LLM tokens:** ~{}k\n\n",
        report.files_analyzed, report.total_lines,
        report.total_estimated_tokens / 1000
    ));
    if report.files_skipped > 0 {
        out.push_str(&format!(
            "> **Warning:** {} files skipped due to parse errors.\n\n",
            report.files_skipped
        ));
    }

    out.push_str("## Global Scores (0-100, higher = worse)\n\n");
    out.push_str("| Metric | Score | Level | Meaning |\n");
    out.push_str("|--------|-------|-------|--------|\n");
    out.push_str(&format!(
        "| **AI Friction** | {}/100 | {} | Overall AI-friction from all detected patterns |\n",
        gs.ai_friction_score, score_level(gs.ai_friction_score)
    ));
    out.push_str(&format!(
        "| **Context Waste** | {}/100 | {} | Token budget wasted by oversized files |\n",
        gs.context_waste_score, score_level(gs.context_waste_score)
    ));
    out.push_str(&format!(
        "| **Structural Entropy** | {}/100 | {} | Noise from coupling, dead code, re-exports, duplication |\n\n",
        gs.structural_entropy_score, score_level(gs.structural_entropy_score)
    ));

    out.push_str(&format!(
        "> **Estimated context waste:** ~{}% of agent context capacity (~{:.1}x Claude sessions)\n\n",
        gs.estimated_waste_pct,
        gs.context_waste_ratio
    ));

    out.push_str("## How to Interpret These Scores\n\n");
    out.push_str("- **0-19:** Healthy — your codebase is well-structured for AI tooling.\n");
    out.push_str("- **20-39:** Moderate — some patterns will slow down AI-assisted edits.\n");
    out.push_str("- **40-69:** Concerning — AI-assisted development will be noticeably degraded.\n");
    out.push_str("- **70-100:** Critical — significant refactoring recommended.\n\n");

    out.push_str("---\n\n");

    if report.findings.is_empty() {
        out.push_str("## Detected Patterns\n\nNo issues found.\n");
        return out;
    }

    out.push_str("## Detected Patterns\n\n");
    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());

    for (i, f) in sorted.iter().enumerate() {
        let tok = f
            .estimated_tokens
            .map(|t| format!(" (~{}k tokens)", t / 1000))
            .unwrap_or_default();
        out.push_str(&format!(
            "### {}. [{}] {}\n\n**Kind:** {}  \n**Summary:** {}{}  \n\n",
            i + 1,
            f.severity.label(),
            f.path.display(),
            f.kind.label(),
            f.summary,
            tok
        ));
        if !f.reasons.is_empty() {
            out.push_str("**Reasons:**\n");
            for r in &f.reasons {
                out.push_str(&format!("- {}\n", r));
            }
            out.push('\n');
        }
        out.push_str(&format!(
            "**Next step:** {}\n\n---\n\n",
            f.suggested_next_step
        ));
    }

    out.push_str("## Priority Cleanup Targets (fix these first)\n\n");
    for f in sorted
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .take(5)
    {
        out.push_str(&format!("- {}\n", f.suggested_next_step));
    }
    out
}

fn score_level(score: u32) -> &'static str {
    if score >= 70 {
        "CRITICAL"
    } else if score >= 40 {
        "MODERATE"
    } else if score >= 20 {
        "ELEVATED"
    } else {
        "LOW"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Finding, FindingKind, GlobalScore, Severity};
    use std::path::PathBuf;

    fn mkreport(findings: Vec<Finding>) -> Report {
        Report {
            findings,
            global_score: GlobalScore {
                ai_friction_score: 42,
                context_waste_score: 30,
                structural_entropy_score: 15,
                context_waste_ratio: 0.16,
                estimated_waste_pct: 3,
            reasoning_complexity_score: 0,
            },
            files_analyzed: 5,
            files_skipped: 0,
            total_lines: 2000,
            total_estimated_tokens: 20000,
            errors: vec![],
            version: "0.0.0".to_string(),
        }
    }

    #[test]
    fn has_header() {
        assert!(render(&mkreport(vec![])).contains("# repo-gc: AI Context Efficiency Report"));
    }
    #[test]
    fn has_scores() {
        assert!(render(&mkreport(vec![])).contains("42/100"));
    }
    #[test]
    fn findings_in_output() {
        let r = mkreport(vec![Finding {
            id: "cb-001".into(),
            kind: FindingKind::ContextBomb,
            severity: Severity::High,
            confidence: 0.9,
            path: PathBuf::from("src/big.rs"),
            summary: "Large".into(),
            reasons: vec![],
            evidence: vec![],
            suggested_next_step: "Split".into(),
            estimated_tokens: Some(8000),
        }]);
        let md = render(&r);
        assert!(md.contains("src/big.rs") && md.contains("HIGH"));
    }
    #[test]
    fn skip_count_in_output() {
        let mut r = mkreport(vec![]);
        r.files_skipped = 3;
        assert!(render(&r).contains("3 files skipped due to parse errors"));
    }
}
