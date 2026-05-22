//! Branch density heuristic.
//!
//! Measures cyclomatic complexity per function. High branch density degrades
//! LLM reasoning performance (RE2-Bench: 51.5% perf drop from low to high
//! complexity).
//!
//! Logic: branch_count / max(function_count, 1) > limit → finding.
//! Severity scales with ratio: >= limit*3 → Critical, >= limit*2 → High,
//! >= limit → Medium.

use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let limit = threshold.branch_density_limit();
    let fn_count = std::cmp::max(structure.function_count, 1);
    let ratio = structure.branch_count as f64 / fn_count as f64;

    if ratio <= limit as f64 {
        return None;
    }

    let severity = if ratio >= (limit as f64) * 3.0 {
        Severity::Critical
    } else if ratio >= (limit as f64) * 2.0 {
        Severity::High
    } else {
        Severity::Medium
    };

    *counter += 1;
    Some(Finding {
        id: format!("bd-{:03}", counter),
        kind: FindingKind::BranchDensity,
        severity,
        confidence: 0.85,
        path: file.relative_path.clone(),
        summary: String::new(),
        reasons: vec![],
        evidence: vec![
            format!("branch_count: {}", structure.branch_count),
            format!("function_count: {}", fn_count),
            format!("avg_branches_per_fn: {:.2}", ratio),
            format!("limit: {}", limit),
        ],
        suggested_next_step: String::new(),
        estimated_tokens: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn f(name: &str) -> RustFile {
        RustFile {
            path: PathBuf::from(name),
            relative_path: PathBuf::from(name),
            package_name: "krate".to_string(),
            size_bytes: 100,
            line_count: 10,
        }
    }

    fn s(branch_count: usize, function_count: usize) -> FileStructure {
        FileStructure {
            branch_count,
            function_count,
            path: PathBuf::from("src/test.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        // Normal limit is 12; ratio=1.0 (12/12) is not above limit
        let r = analyze(&f("src/test.rs"), &s(12, 12), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_at_exact_limit() {
        // Normal limit=12, branch=12, fn=1 => ratio=12 => not above limit
        let r = analyze(&f("src/test.rs"), &s(12, 1), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_below_strict_limit() {
        // Strict limit=8, ratio=8/1=8 => not above limit
        let r = analyze(&f("src/test.rs"), &s(8, 1), &Threshold::Strict, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn finding_above_limit() {
        // Normal limit=12, branch=25, fn=2 => ratio=12.5 => above limit
        let r = analyze(&f("src/test.rs"), &s(25, 2), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::BranchDensity);
        assert_eq!(r.confidence, 0.85);
    }

    #[test]
    fn severity_medium_at_limit() {
        // Normal limit=12, ratio=13/1=13 → >=12 but <24 → Medium
        let r = analyze(&f("src/test.rs"), &s(13, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_double_limit() {
        // Normal limit=12, ratio=24/1=24 → >=24 but <36 → High
        let r = analyze(&f("src/test.rs"), &s(24, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn severity_critical_at_triple_limit() {
        // Normal limit=12, ratio=36/1=36 → >=36 → Critical
        let r = analyze(&f("src/test.rs"), &s(36, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn avoids_division_by_zero() {
        // function_count=0 should be treated as 1
        let r = analyze(&f("src/test.rs"), &s(25, 0), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::BranchDensity);
        // evidence should show function_count: 1
        assert!(r.evidence.iter().any(|e| e.contains("function_count: 1")));
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f("src/test.rs"), &s(25, 1), &Threshold::Normal, &mut c);
        analyze(&f("src/test.rs"), &s(25, 1), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
