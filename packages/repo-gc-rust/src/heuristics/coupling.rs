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

    let severity = if fan_in >= in_limit * 3 || fan_out >= out_limit * 2 {
        Severity::High
    } else {
        Severity::Medium
    };

    let mut reasons = vec![];
    if fan_in >= in_limit {
        reasons.push(format!(
            "Fan-in: {} modules import this (limit: {})",
            fan_in, in_limit
        ));
    }
    if fan_out >= out_limit {
        reasons.push(format!(
            "Fan-out: imports {} modules (limit: {})",
            fan_out, out_limit
        ));
    }

    *counter += 1;
    Some(Finding {
        id: format!("ch-{:03}", counter),
        kind: FindingKind::CouplingHotspot,
        severity,
        confidence: 0.85,
        path: file.relative_path.clone(),
        summary: format!(
            "Dependency concentration — fan-in={}, fan-out={} increases LLM reasoning overhead",
            fan_in, fan_out
        ),
        reasons,
        evidence: vec![
            format!("fan_in: {}", fan_in),
            format!("fan_out: {}", fan_out),
        ],
        suggested_next_step: format!(
            "Extract an interface to decouple {} from its dependents",
            file.relative_path.display()
        ),
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
    fn very_high_is_high_severity() {
        assert_eq!(
            analyze(
                &mkfile("src/hub.rs"),
                &mkgraph(35, 5, "src/hub.rs"),
                &Threshold::Normal,
                &mut 0
            )
            .unwrap()
            .severity,
            Severity::High
        );
    }
}
