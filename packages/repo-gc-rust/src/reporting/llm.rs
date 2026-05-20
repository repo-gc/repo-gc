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
            // evidence: ["pub_use_count: N", "total_reexported_items: N", "has_wildcard: B"]
            let pu = ev_val(&f.evidence, "pub_use_count");
            let ti = ev_val(&f.evidence, "total_reexported_items");
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
            // evidence: ["imported but unreferenced: name", ...]
            let n = f.evidence.len();
            let names: Vec<&str> = f
                .evidence
                .iter()
                .filter_map(|e| e.strip_prefix("imported but unreferenced: "))
                .take(4)
                .collect();
            format!("{n}: {}", names.join(","))
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
            },
            files_analyzed: 50,
            files_skipped: 0,
            total_lines: 10000,
            total_estimated_tokens: 80000,
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
            },
            files_analyzed: 1,
            files_skipped: 0,
            total_lines: 600,
            total_estimated_tokens: 5000,
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
            },
            files_analyzed: 1,
            files_skipped: 0,
            total_lines: 500,
            total_estimated_tokens: 4000,
        }
    }
}
