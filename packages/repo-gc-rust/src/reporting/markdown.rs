use crate::types::{Report, Severity};

pub fn render(report: &Report) -> String {
    let mut out = String::from("# Repository Hygiene Report\n\n");
    let skip = if report.files_skipped > 0 {
        format!(
            "  |  **Skipped:** {} (parse errors)",
            report.files_skipped
        )
    } else {
        String::new()
    };
    out.push_str(&format!(
        "**Files:** {}  |  **Lines:** {}  |  **Est. tokens:** ~{}k{}\n\n",
        report.files_analyzed,
        report.total_lines,
        report.total_estimated_tokens / 1000,
        skip
    ));

    let gs = &report.global_score;
    out.push_str(&format!(
        "## Global Scores\n\n| Metric | Score |\n|--------|-------|\n\
         | AI Hostility | {}/100 |\n| Context Waste | {}/100 |\n| Entropy | {}/100 |\n\n",
        gs.ai_hostility_score, gs.context_waste_score, gs.entropy_score
    ));

    if report.findings.is_empty() {
        out.push_str("## Findings\n\nNo issues found.\n");
        return out;
    }

    out.push_str("## Findings\n\n");
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

    out.push_str("## Recommended Cleanup Targets\n\n");
    for f in sorted
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .take(5)
    {
        out.push_str(&format!("- {}\n", f.suggested_next_step));
    }
    out
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
                ai_hostility_score: 42,
                context_waste_score: 30,
                entropy_score: 15,
            },
            files_analyzed: 5,
            files_skipped: 0,
            total_lines: 2000,
            total_estimated_tokens: 20000,
        }
    }

    #[test]
    fn has_header() {
        assert!(render(&mkreport(vec![])).contains("# Repository Hygiene Report"));
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
        assert!(render(&r).contains("3 (parse errors)") || render(&r).contains("Skipped: 3"));
    }
}
