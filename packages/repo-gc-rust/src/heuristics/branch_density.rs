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
use crate::heuristics::common::severity_scale;
use crate::parsing::FileStructure;
use crate::types::{next_finding_id, Finding, FindingKind};

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

    let severity = severity_scale(ratio, limit as f64);

    Some(Finding::new(
        next_finding_id("bd", counter),
        FindingKind::BranchDensity,
        severity,
        0.85,
        file.relative_path.clone(),
        vec![
            format!("branch_count: {}", structure.branch_count),
            format!("function_count: {}", fn_count),
            format!("avg_branches_per_fn: {:.2}", ratio),
            format!("limit: {}", limit),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heuristics::test_utils::test_file;
    use crate::types::Severity;
    use std::path::PathBuf;

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
        let r = analyze(&test_file("src/test.rs"), &s(12, 12), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_at_exact_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(12, 1), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_below_strict_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(8, 1), &Threshold::Strict, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn finding_above_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(25, 2), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::BranchDensity);
        assert_eq!(r.confidence, 0.85);
    }

    #[test]
    fn severity_medium_at_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(13, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_double_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(24, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn severity_critical_at_triple_limit() {
        let r = analyze(&test_file("src/test.rs"), &s(36, 1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn avoids_division_by_zero() {
        let r = analyze(&test_file("src/test.rs"), &s(25, 0), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::BranchDensity);
        assert!(r.evidence.iter().any(|e| e.contains("function_count: 1")));
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&test_file("src/test.rs"), &s(25, 1), &Threshold::Normal, &mut c);
        analyze(&test_file("src/test.rs"), &s(25, 1), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
