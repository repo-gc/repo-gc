//! Stringly-typed heuristic.
//!
//! Detects magic string comparisons — string literals used in comparisons and
//! match arms where enums or constants should be used instead.

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
    let limit = threshold.string_comparison_limit();
    if structure.string_comparison_count <= limit {
        return None;
    }

    let severity = if structure.string_comparison_count >= limit * 3 {
        Severity::High
    } else if structure.string_comparison_count >= limit * 2 {
        Severity::Medium
    } else {
        Severity::Low
    };

    Some(Finding::new(
        next_finding_id("st", counter),
        FindingKind::StringlyTyped,
        severity,
        0.65,
        file.relative_path.clone(),
        vec![
            format!("string_comparison_count: {}", structure.string_comparison_count),
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
            string_comparison_count: count,
            path: PathBuf::from("src/lib.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_at_normal_limit() {
        // Normal limit is 10; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(10), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_strict_limit() {
        // Strict limit is 5; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(5), &Threshold::Strict, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_relaxed_limit() {
        // Relaxed limit is 15; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(15), &Threshold::Relaxed, &mut 0).is_none());
    }

    #[test]
    fn finding_above_limit() {
        let r = analyze(&f("src/lib.rs"), &s(11), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::StringlyTyped);
        assert!((r.confidence - 0.65).abs() < f32::EPSILON);
    }

    #[test]
    fn severity_low_at_limit_plus_one() {
        // Normal limit=10, count=11 → above limit but < limit*2 → Low
        let r = analyze(&f("src/lib.rs"), &s(11), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Low);
    }

    #[test]
    fn severity_medium_at_limit_double() {
        // Normal limit=10, count=20 → >= limit*2 → Medium
        let r = analyze(&f("src/lib.rs"), &s(20), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_limit_triple() {
        // Normal limit=10, count=30 → >= limit*3 → High
        let r = analyze(&f("src/lib.rs"), &s(30), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn evidence_contains_count_and_limit() {
        let r = analyze(&f("src/lib.rs"), &s(15), &Threshold::Normal, &mut 0).unwrap();
        assert!(
            r.evidence.iter().any(|e| e.contains("string_comparison_count: 15")),
            "evidence should include actual count"
        );
        assert!(
            r.evidence.iter().any(|e| e.contains("limit: 10")),
            "evidence should include threshold limit"
        );
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f("src/lib.rs"), &s(11), &Threshold::Normal, &mut c);
        analyze(&f("src/lib.rs"), &s(11), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
