use crate::types::{Finding, FindingKind, Report};

/// Token-optimized TSV output for LLM consumption.
/// Short codes, no prose, evidence parsed into compact notation.
///
/// Format:
///   Line 1: summary stats
///   Line 2: column headers (if findings > 0)
///   Lines 3+: findings
pub fn render(report: &Report) -> String {
    let gs = &report.global_score;
    let mut out = format!(
        "repo-gc\tfric={}\twaste={}\tent={}\tpct={}\trat={:.1}\tf={}\tln={}\ttok={}k\n",
        gs.ai_friction_score,
        gs.context_waste_score,
        gs.structural_entropy_score,
        gs.estimated_waste_pct,
        gs.context_waste_ratio,
        report.files_analyzed,
        report.total_lines,
        report.total_estimated_tokens / 1000,
    );

    if report.findings.is_empty() {
        return out;
    }

    out.push_str("id\tsev\tkind\tpath\ttok\tsummary\tnext\n");

    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());

    for f in &sorted {
        let tok = f
            .estimated_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "-".into());
        let summary = compact_summary(f);
        let next = compact_next(f);
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            f.id,
            f.severity.llm_label(),
            f.kind.llm_label(),
            f.path.display(),
            tok,
            summary,
            next,
        ));
    }
    out
}

fn compact_summary(f: &Finding) -> String {
    match f.kind {
        FindingKind::ContextBomb => {
            // evidence: ["line_count: N", "estimated_tokens: N"]
            let lc = ev_val(&f.evidence, "line_count");
            let tk = ev_val(&f.evidence, "estimated_tokens");
            let mut s = format!("{lc}ln/{tk}tk");
            // reasons may have fn/impl counts
            for r in &f.reasons {
                if r.contains("functions defined") {
                    if let Some(n) = r.split_whitespace().next() {
                        s.push_str(&format!(" +{n}fn"));
                    }
                } else if r.contains("impl blocks") {
                    if let Some(n) = r.split_whitespace().next() {
                        s.push_str(&format!(" +{n}impl"));
                    }
                }
            }
            s
        }
        FindingKind::CouplingHotspot => {
            // evidence: ["fan_in: N", "fan_out: N", "instability: X.XXX", "pattern: label"]
            let fi = ev_val(&f.evidence, "fan_in");
            let fo = ev_val(&f.evidence, "fan_out");
            let istab = ev_val(&f.evidence, "instability");
            let pat = ev_str(&f.evidence, "pattern");
            format!("in={fi} out={fo} I={istab} {pat}")
        }
        FindingKind::DeadWeight => {
            // evidence: ["module_path: X", "line_count: N"]
            let mp = ev_str(&f.evidence, "module_path");
            let lc = ev_val(&f.evidence, "line_count");
            format!("{lc}ln mod={mp}")
        }
        FindingKind::ReexportEntropy => {
            // evidence: ["reexport_count: N", "total_items: N", "has_wildcard: B"]
            let pu = ev_val(&f.evidence, "reexport_count");
            let ti = ev_val(&f.evidence, "total_items");
            let wc = ev_str(&f.evidence, "has_wildcard");
            let w = if wc == "true" { " +*" } else { "" };
            format!("{ti}sym/{pu}pu{w}")
        }
        FindingKind::CodeDuplication => {
            // evidence: ["file_path :: fn_name", ...]
            let files = f.evidence.len();
            let fn_name = f
                .evidence
                .first()
                .and_then(|e| e.split(" :: ").nth(1))
                .unwrap_or("?");
            format!("fn:{fn_name} x{files}")
        }
        FindingKind::UnusedImport => {
            // evidence: ["unused_count: N", "preview: name1, name2, ..."]
            let n = ev_val(&f.evidence, "unused_count");
            let names = ev_str(&f.evidence, "preview");
            format!("{n}: {names}")
        }
        FindingKind::ErrorSwallow => {
            let n = ev_val(&f.evidence, "empty_catch_count");
            format!("{n} empty catch")
        }
        FindingKind::ImportDiversity => {
            let dc = ev_val(&f.evidence, "domain_count");
            let limit = ev_val(&f.evidence, "limit");
            let domains = ev_str(&f.evidence, "domains");
            format!("{dc}dom (limit:{limit}): {domains}")
        }
        FindingKind::BranchDensity => {
            let bc = ev_val(&f.evidence, "branch_count");
            let fc = ev_val(&f.evidence, "function_count");
            let avg = ev_val(&f.evidence, "avg_branches_per_fn");
            format!("{avg}br/fn ({bc}br, {fc}fn)")
        }
        FindingKind::DeepNesting => {
            let depth = ev_val(&f.evidence, "max_depth");
            let limit = ev_val(&f.evidence, "limit");
            format!("{depth}nest (limit:{limit})")
        }
        FindingKind::DangerousPattern => {
            let count = ev_val(&f.evidence, "dangerous_pattern_count");
            let limit = ev_val(&f.evidence, "limit");
            format!("{count}dang (limit:{limit})")
        }
        FindingKind::CommentRatio => {
            let ratio = ev_val(&f.evidence, "ratio");
            let dir = ev_str(&f.evidence, "direction");
            format!("{ratio} {dir}")
        }
        FindingKind::StringlyTyped => {
            let count = ev_val(&f.evidence, "string_comparison_count");
            let limit = ev_val(&f.evidence, "limit");
            format!("{count}str (limit:{limit})")
        }
        FindingKind::NamingEntropy => {
            let dc = ev_str(&f.evidence, "dominant_convention");
            let mc = ev_val(&f.evidence, "mixed_count");
            format!("{mc}conventions dominant={dc}")
        }
        FindingKind::TypeComplexity => {
            let depth = ev_val(&f.evidence, "max_type_depth");
            let limit = ev_val(&f.evidence, "limit");
            format!("{depth}nest (limit:{limit})")
        }
        FindingKind::ImplicitControl => {
            let ratio = ev_val(&f.evidence, "ratio");
            let limit = ev_val(&f.evidence, "limit");
            format!("{ratio}dec/fn (limit:{limit})")
        }
    }
}

fn compact_next(f: &Finding) -> String {
    match f.kind {
        FindingKind::ContextBomb => {
            let lc = ev_val(&f.evidence, "line_count");
            format!("split <{lc}ln")
        }
        FindingKind::CouplingHotspot => {
            match ev_str(&f.evidence, "pattern").as_str() {
                "api" => "verify api",
                "orch" => "split deps",
                _ => "decouple",
            }
            .into()
        }
        FindingKind::DeadWeight => "rm or re-export".into(),
        FindingKind::ReexportEntropy => "flatten re-exports".into(),
        FindingKind::CodeDuplication => "DRY: shared util".into(),
        FindingKind::UnusedImport => "rm imports".into(),
        FindingKind::ErrorSwallow => "handle errors".into(),
        FindingKind::BranchDensity => "decompose fns".into(),
        FindingKind::DeepNesting => "flatten nesting".into(),
        FindingKind::ImportDiversity => "split by domain".into(),
        FindingKind::DangerousPattern => "refactor unsafe patterns".into(),
        FindingKind::CommentRatio => {
            match ev_str(&f.evidence, "direction").as_str() {
                "sparse" => "add comments",
                _ => "trim comments",
            }
            .into()
        }
        FindingKind::StringlyTyped => "use enums".into(),
        FindingKind::NamingEntropy => "unify naming conventions".into(),
        FindingKind::TypeComplexity => "simplify types".into(),
        FindingKind::ImplicitControl => "minimize decorators".into(),
    }
}

fn ev_val(evidence: &[String], key: &str) -> String {
    evidence
        .iter()
        .find_map(|e| e.strip_prefix(&format!("{key}: ")))
        .unwrap_or("-")
        .to_string()
}

fn ev_str(evidence: &[String], key: &str) -> String {
    ev_val(evidence, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Finding, FindingKind, GlobalScore, Severity};
    use std::path::PathBuf;

    #[test]
    fn llm_header_contains_short_keys() {
        let r = Report {
            findings: vec![],
            global_score: GlobalScore {
                ai_friction_score: 72,
                context_waste_score: 40,
                structural_entropy_score: 30,
                context_waste_ratio: 1.5,
                estimated_waste_pct: 5,
            reasoning_complexity_score: 0,
            },
            files_analyzed: 50,
            files_skipped: 0,
            total_lines: 10000,
            total_estimated_tokens: 80000,
            errors: vec![],
            version: "0.0.0".to_string(),
        };
        let out = render(&r);
        assert!(out.contains("fric=72"), "missing fric key");
        assert!(out.contains("ent=30"), "missing ent key");
    }

    #[test]
    fn llm_uses_short_severity_labels() {
        let f = Finding {
            id: "x".into(),
            kind: FindingKind::ContextBomb,
            severity: Severity::High,
            confidence: 1.0,
            path: PathBuf::from("src/x.rs"),
            summary: "".into(),
            reasons: vec![],
            evidence: vec!["line_count: 600".into(), "estimated_tokens: 5000".into()],
            suggested_next_step: "".into(),
            estimated_tokens: Some(5000),
        };
        let r = Report {
            findings: vec![f],
            global_score: GlobalScore {
                ai_friction_score: 50,
                context_waste_score: 50,
                structural_entropy_score: 0,
                context_waste_ratio: 0.0,
                estimated_waste_pct: 0,
            reasoning_complexity_score: 0,
            },
            files_analyzed: 1,
            files_skipped: 0,
            total_lines: 600,
            total_estimated_tokens: 5000,
            errors: vec![],
            version: "0.0.0".to_string(),
        };
        let out = render(&r);
        assert!(out.contains("\tH\tOVS\t"), "expected short labels H and OVS");
    }

    fn coupling_finding(evidence: Vec<&str>) -> Finding {
        Finding {
            id: "ch-001".into(),
            kind: FindingKind::CouplingHotspot,
            severity: Severity::Medium,
            confidence: 0.65,
            path: PathBuf::from("src/api.rs"),
            summary: "test".into(),
            reasons: vec![],
            evidence: evidence.into_iter().map(String::from).collect(),
            suggested_next_step: "test".into(),
            estimated_tokens: None,
        }
    }

    #[test]
    fn coupling_compact_summary_api_hub() {
        let f = coupling_finding(vec![
            "fan_in: 513",
            "fan_out: 8",
            "instability: 0.015",
            "pattern: api",
        ]);
        let r = make_report(vec![f]);
        let out = render(&r);
        assert!(out.contains("in=513 out=8 I=0.015 api"), "expected api pattern in summary: {out}");
    }

    #[test]
    fn coupling_compact_summary_god_module() {
        let f = coupling_finding(vec![
            "fan_in: 50",
            "fan_out: 40",
            "instability: 0.444",
            "pattern: god",
        ]);
        let r = make_report(vec![f]);
        let out = render(&r);
        assert!(out.contains("in=50 out=40 I=0.444 god"), "expected god pattern in summary: {out}");
    }

    #[test]
    fn coupling_compact_next_api_verify() {
        let f = coupling_finding(vec![
            "fan_in: 513", "fan_out: 8", "instability: 0.015", "pattern: api",
        ]);
        let r = make_report(vec![f]);
        let out = render(&r);
        assert!(out.contains("verify api"), "expected 'verify api' next step: {out}");
    }

    #[test]
    fn coupling_compact_next_orch_split() {
        let f = coupling_finding(vec![
            "fan_in: 2", "fan_out: 20", "instability: 0.909", "pattern: orch",
        ]);
        let r = make_report(vec![f]);
        let out = render(&r);
        assert!(out.contains("split deps"), "expected 'split deps' next step: {out}");
    }

    #[test]
    fn coupling_compact_next_god_decouple() {
        let f = coupling_finding(vec![
            "fan_in: 50", "fan_out: 40", "instability: 0.444", "pattern: god",
        ]);
        let r = make_report(vec![f]);
        let out = render(&r);
        assert!(out.contains("decouple"), "expected 'decouple' next step: {out}");
    }

    fn make_report(findings: Vec<Finding>) -> Report {
        Report {
            findings,
            global_score: GlobalScore {
                ai_friction_score: 50,
                context_waste_score: 50,
                structural_entropy_score: 0,
                context_waste_ratio: 0.0,
                estimated_waste_pct: 0,
            reasoning_complexity_score: 0,
            },
            files_analyzed: 1,
            files_skipped: 0,
            total_lines: 500,
            total_estimated_tokens: 4000,
            errors: vec![],
            version: "0.0.0".to_string(),
        }
    }
}
