//! Deep nesting heuristic — measures maximum control-flow nesting depth per file.
//!
//! Deeply nested code increases cognitive load for LLMs; they skip condition
//! bodies and miss deeply nested blocks (RE2-Bench: significant perf drop at
//! depths > 4).

use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::heuristics::common::severity_scale_offset;
use crate::parsing::FileStructure;
use crate::types::{next_finding_id, Finding, FindingKind};

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let limit = threshold.nesting_depth_limit();
    if structure.max_nesting_depth <= limit {
        return None;
    }

    let depth = structure.max_nesting_depth;
    let severity = severity_scale_offset(depth, limit, 2, 4);

    Some(Finding::new(
        next_finding_id("dn", counter),
        FindingKind::DeepNesting,
        severity,
        0.85,
        file.relative_path.clone(),
        vec![
            format!("max_depth: {}", depth),
            format!("limit: {}", limit),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;
    use std::path::PathBuf;

    fn f() -> RustFile {
        RustFile {
            path: PathBuf::from("src/nested.rs"),
            relative_path: PathBuf::from("src/nested.rs"),
            package_name: "krate".to_string(),
            size_bytes: 1000,
            line_count: 50,
        }
    }
    fn s(depth: usize) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/nested.rs"),
            max_nesting_depth: depth,
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        assert!(analyze(&f(), &s(5), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn medium_at_limit_plus_one() {
        // Normal limit=6, depth=7 → exceeds limit → Medium
        let r = analyze(&f(), &s(7), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
        assert_eq!(r.confidence, 0.85);
    }

    #[test]
    fn high_at_limit_plus_2() {
        let r = analyze(&f(), &s(8), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn critical_at_limit_plus_4() {
        let r = analyze(&f(), &s(10), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn id_uses_dn_prefix() {
        let r = analyze(&f(), &s(7), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.id.starts_with("dn-"));
    }

    #[test]
    fn no_finding_at_strict_limit() {
        // Strict limit=4, depth=4 → no finding
        assert!(analyze(&f(), &s(4), &Threshold::Strict, &mut 0).is_none());
    }

    #[test]
    fn finding_at_relaxed_limit_plus_one() {
        // Relaxed limit=8, depth=9 → Medium
        let r = analyze(&f(), &s(9), &Threshold::Relaxed, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f(), &s(7), &Threshold::Normal, &mut c);
        analyze(&f(), &s(7), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }

    #[test]
    fn evidence_contains_max_depth_and_limit() {
        let r = analyze(&f(), &s(7), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.evidence.iter().any(|e| e.starts_with("max_depth: 7")));
        assert!(r.evidence.iter().any(|e| e.starts_with("limit: 6")));
    }
}
