//! Error swallowing heuristic.
//!
//! Detects discarded error handling: empty `Err(_)` match arms, empty
//! `if let Err(_) = expr {}` blocks, and `.ok()` calls that discard
//! the error variant. Since `syn` has no type resolution, detection is
//! best-effort with lower confidence.

use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{next_finding_id, Finding, FindingKind, Severity};

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let limit = threshold.empty_catch_limit();
    if structure.empty_catch_count <= limit {
        return None;
    }

    let severity = if structure.empty_catch_count >= 5 {
        Severity::High
    } else if structure.empty_catch_count >= 3 {
        Severity::Medium
    } else {
        Severity::Low
    };

    Some(Finding::new(
        next_finding_id("es", counter),
        FindingKind::ErrorSwallow,
        severity,
        0.4, // Lower: syn has no type resolution
        file.relative_path.clone(),
        vec![
            format!("empty_catch_count: {}", structure.empty_catch_count),
            format!("limit: {}", limit),
            format!("preview: {}", file.relative_path.display()),
        ],
    ))
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

    fn s(count: usize) -> FileStructure {
        FileStructure {
            empty_catch_count: count,
            path: PathBuf::from("src/test.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_at_normal_limit() {
        // Normal limit is 2; count == limit means no finding
        assert!(analyze(&f("src/test.rs"), &s(2), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_strict_limit() {
        // Strict limit is 1; count == limit means no finding
        assert!(analyze(&f("src/test.rs"), &s(1), &Threshold::Strict, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_relaxed_limit() {
        // Relaxed limit is 3; count == limit means no finding
        assert!(analyze(&f("src/test.rs"), &s(3), &Threshold::Relaxed, &mut 0).is_none());
    }

    #[test]
    fn finding_above_limit() {
        let r = analyze(&f("src/test.rs"), &s(3), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::ErrorSwallow);
        assert_eq!(r.confidence, 0.4);
    }

    #[test]
    fn severity_low_at_two_strict() {
        // Strict limit=1, count=2 → above limit but < 3 → Low
        let r = analyze(&f("src/test.rs"), &s(2), &Threshold::Strict, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Low);
    }

    #[test]
    fn severity_medium_at_three() {
        let r = analyze(&f("src/test.rs"), &s(3), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_five() {
        let r = analyze(&f("src/test.rs"), &s(5), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn evidence_contains_count_and_limit() {
        let r = analyze(&f("src/test.rs"), &s(5), &Threshold::Normal, &mut 0).unwrap();
        assert!(
            r.evidence.iter().any(|e| e.contains("empty_catch_count: 5")),
            "evidence should include actual count"
        );
        assert!(
            r.evidence.iter().any(|e| e.contains("limit: 2")),
            "evidence should include threshold limit"
        );
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f("src/test.rs"), &s(3), &Threshold::Normal, &mut c);
        analyze(&f("src/test.rs"), &s(3), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
