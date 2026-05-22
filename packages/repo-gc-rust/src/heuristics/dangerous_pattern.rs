//! Dangerous pattern heuristic.
//!
//! Detects risky code patterns (unsafe, unwrap, expect, transmute) that force
//! LLMs to consider a broader state space. Confidence is lower for Rust because
//! `.unwrap()` and `unsafe` have legitimate uses in low-level code.
//!
//! Test-file suppression ships in the parser (extractor.rs) — `.unwrap()` and
//! `.expect()` are not counted in test files, and the total count is halved.

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
    let limit = threshold.dangerous_pattern_limit();
    if structure.dangerous_pattern_count <= limit {
        return None;
    }

    let severity = if structure.dangerous_pattern_count >= limit * 3 {
        Severity::Critical
    } else if structure.dangerous_pattern_count >= limit * 2 {
        Severity::High
    } else {
        Severity::Medium
    };

    *counter += 1;
    Some(Finding {
        id: format!("dp-{:03}", counter),
        kind: FindingKind::DangerousPattern,
        severity,
        confidence: 0.60, // Lower: Rust has legitimate uses for unsafe/unwrap
        path: file.relative_path.clone(),
        summary: String::new(),
        reasons: vec![],
        evidence: vec![
            format!("dangerous_pattern_count: {}", structure.dangerous_pattern_count),
            format!("limit: {}", limit),
            format!("preview: {}", file.relative_path.display()),
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

    fn s(count: usize) -> FileStructure {
        FileStructure {
            dangerous_pattern_count: count,
            path: PathBuf::from("src/lib.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_at_normal_limit() {
        // Normal limit is 5; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(5), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_strict_limit() {
        // Strict limit is 3; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(3), &Threshold::Strict, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_relaxed_limit() {
        // Relaxed limit is 10; count == limit means no finding
        assert!(analyze(&f("src/lib.rs"), &s(10), &Threshold::Relaxed, &mut 0).is_none());
    }

    #[test]
    fn finding_above_limit() {
        let r = analyze(&f("src/lib.rs"), &s(6), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::DangerousPattern);
        assert!((r.confidence - 0.60).abs() < f32::EPSILON);
    }

    #[test]
    fn severity_medium_at_limit_plus_one() {
        // Normal limit=5, count=6 → above limit but < limit*2 → Medium
        let r = analyze(&f("src/lib.rs"), &s(6), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_limit_double() {
        // Normal limit=5, count=10 → >= limit*2 → High
        let r = analyze(&f("src/lib.rs"), &s(10), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn severity_critical_at_limit_triple() {
        // Normal limit=5, count=15 → >= limit*3 → Critical
        let r = analyze(&f("src/lib.rs"), &s(15), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn evidence_contains_count_and_limit() {
        let r = analyze(&f("src/lib.rs"), &s(8), &Threshold::Normal, &mut 0).unwrap();
        assert!(
            r.evidence.iter().any(|e| e.contains("dangerous_pattern_count: 8")),
            "evidence should include actual count"
        );
        assert!(
            r.evidence.iter().any(|e| e.contains("limit: 5")),
            "evidence should include threshold limit"
        );
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f("src/lib.rs"), &s(6), &Threshold::Normal, &mut c);
        analyze(&f("src/lib.rs"), &s(6), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
