use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::graph::ImportGraph;
use crate::types::{Finding, FindingKind, Severity};

pub fn analyze(
    file: &RustFile,
    graph: &ImportGraph,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let fan_in = graph.get_fan_in(&file.path);
    let fan_out = graph.get_fan_out(&file.path);
    let in_limit = threshold.fan_in_limit();
    let out_limit = threshold.fan_out_limit();

    if fan_in < in_limit && fan_out < out_limit {
        return None;
    }

    let i = if fan_in + fan_out > 0 {
        fan_out as f32 / (fan_in + fan_out) as f32
    } else {
        0.0
    };

    let fan_in_over = fan_in >= in_limit;
    let fan_out_over = fan_out >= out_limit;

    let (severity, confidence, summary, suggested_next_step, pattern_label, i_interpretation) =
        match (fan_in_over, fan_out_over) {
            (true, false) => {
                // API/dispatch hub: high fan-in, low fan-out — intentionally stable
                (
                    Severity::Medium,
                    0.65,
                    format!(
                        "Wide API surface — fan-in={fi}, fan-out={fo}, I={i:.2}. This looks like an intentional API/dispatch hub — verify it's not accidental coupling.",
                        fi = fan_in,
                        fo = fan_out,
                        i = i,
                    ),
                    format!(
                        "If this is an intentional API/dispatch hub, this is fine — no action needed. Otherwise, split callers across focused interfaces ({} → {} dependents).",
                        file.relative_path.display(),
                        fan_in,
                    ),
                    "api",
                    if i < 0.1 { "very stable (API-like)" } else { "stable (API-like)" },
                )
            }
            (false, true) => {
                // Over-orchestrator: high fan-out, low fan-in — depends on too many peers
                let sev = if fan_out >= out_limit * 3 {
                    Severity::High
                } else {
                    Severity::Medium
                };
                (
                    sev,
                    0.70,
                    format!(
                        "High dependency fan-out ({fo}), low fan-in ({fi}), I={i:.2}. This module imports many peers — consider decomposing.",
                        fi = fan_in,
                        fo = fan_out,
                        i = i,
                    ),
                    "Decompose into focused modules with fewer dependencies each.".into(),
                    "orch",
                    if i > 0.9 { "very unstable (consumer-like)" } else { "unstable (consumer-like)" },
                )
            }
            (true, true) => {
                // God module: both dimensions exceed thresholds
                let sev = if fan_in >= in_limit * 3 || fan_out >= out_limit * 2 {
                    Severity::High
                } else {
                    Severity::Medium
                };
                (
                    sev,
                    0.85,
                    format!(
                        "Dependency concentration — fan-in={fi}, fan-out={fo}, I={i:.2}. High change-impact surface AND high dependency count.",
                        fi = fan_in,
                        fo = fan_out,
                        i = i,
                    ),
                    format!(
                        "Extract an interface to decouple {} from its dependents.",
                        file.relative_path.display()
                    ),
                    "god",
                    "balanced",
                )
            }
            (false, false) => unreachable!(), // caught by guard above
        };

    let mut reasons = vec![];
    if fan_in_over {
        reasons.push(format!(
            "Fan-in: {} modules import this (limit: {})",
            fan_in, in_limit
        ));
    }
    if fan_out_over {
        reasons.push(format!(
            "Fan-out: imports {} modules (limit: {})",
            fan_out, out_limit
        ));
    }
    reasons.push(format!(
        "Instability I={:.3} — {}",
        i, i_interpretation
    ));

    *counter += 1;
    Some(Finding {
        id: format!("ch-{:03}", counter),
        kind: FindingKind::CouplingHotspot,
        severity,
        confidence,
        path: file.relative_path.clone(),
        summary,
        reasons,
        evidence: vec![
            format!("fan_in: {}", fan_in),
            format!("fan_out: {}", fan_out),
            format!("instability: {:.3}", i),
            format!("pattern: {}", pattern_label),
        ],
        suggested_next_step,
        estimated_tokens: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mkfile(p: &str) -> RustFile {
        RustFile {
            path: PathBuf::from(p),
            relative_path: PathBuf::from(p),
            package_name: "krate".to_string(),
            size_bytes: 5000,
            line_count: 200,
        }
    }
    fn mkgraph(fi: usize, fo: usize, p: &str) -> ImportGraph {
        let pb = PathBuf::from(p);
        ImportGraph {
            fan_in: [(pb.clone(), fi)].into(),
            fan_out: [(pb.clone(), fo)].into(),
        }
    }

    #[test]
    fn no_finding_for_low_coupling() {
        assert!(
            analyze(&mkfile("src/x.rs"), &mkgraph(2, 3, "src/x.rs"), &Threshold::Normal, &mut 0)
                .is_none()
        );
    }
    #[test]
    fn flags_high_fan_in() {
        let f = analyze(
            &mkfile("src/core.rs"),
            &mkgraph(15, 2, "src/core.rs"),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert!(f.reasons.iter().any(|r| r.contains("Fan-in")));
    }
    #[test]
    fn flags_high_fan_out() {
        let f = analyze(
            &mkfile("src/god.rs"),
            &mkgraph(2, 20, "src/god.rs"),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert!(f.reasons.iter().any(|r| r.contains("Fan-out")));
    }
    #[test]
    fn fan_in_only_is_always_medium() {
        // fan_in=35, fan_out=5 → fan_in-only → (true, false) → always Medium
        assert_eq!(
            analyze(
                &mkfile("src/hub.rs"),
                &mkgraph(35, 5, "src/hub.rs"),
                &Threshold::Normal,
                &mut 0
            )
            .unwrap()
            .severity,
            Severity::Medium
        );
    }

    #[test]
    fn fan_in_only_even_extreme_stays_medium() {
        // fan_in=500, fan_out=0 → fan-in-only → always Medium regardless of magnitude
        assert_eq!(
            analyze(
                &mkfile("src/dispatch.rs"),
                &mkgraph(500, 0, "src/dispatch.rs"),
                &Threshold::Normal,
                &mut 0
            )
            .unwrap()
            .severity,
            Severity::Medium
        );
    }

    #[test]
    fn god_module_stays_high_severity() {
        // fan_in=35, fan_out=35 → both exceed → god module → High (35 >= 10*3)
        let f = analyze(
            &mkfile("src/god.rs"),
            &mkgraph(35, 35, "src/god.rs"),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert_eq!(f.severity, Severity::High);
        assert_eq!(f.confidence, 0.85);
        assert!(f.summary.contains("I="));
        assert!(f.evidence.iter().any(|e| e.starts_with("pattern: god")));
    }

    #[test]
    fn orchestrator_escalates_to_high_when_extreme() {
        // fan_in=0, fan_out=50, Normal → fan-out-only → 50 >= 45 (15*3) → High
        assert_eq!(
            analyze(
                &mkfile("src/orch.rs"),
                &mkgraph(0, 50, "src/orch.rs"),
                &Threshold::Normal,
                &mut 0
            )
            .unwrap()
            .severity,
            Severity::High
        );
    }

    #[test]
    fn api_hub_has_correct_evidence_and_confidence() {
        let f = analyze(
            &mkfile("src/api.rs"),
            &mkgraph(15, 3, "src/api.rs"),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert_eq!(f.severity, Severity::Medium);
        assert_eq!(f.confidence, 0.65);
        assert!(f.evidence.iter().any(|e| e.starts_with("pattern: api")));
        assert!(f.evidence.iter().any(|e| e.starts_with("instability:")));
        assert!(f.reasons.iter().any(|r| r.contains("Instability")));
    }
}
