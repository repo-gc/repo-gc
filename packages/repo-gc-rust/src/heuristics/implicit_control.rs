//! Implicit control flow heuristic — detects high decorator/middleware density.
//!
//! Counts attributes (Rust's decorator equivalent) on items vs total functions.
//! High ratio means hidden execution paths that LLMs may miss.

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
    let decorator_count = structure.decorator_count;
    let fn_count = std::cmp::max(structure.function_count, 1);
    let ratio = decorator_count as f64 / fn_count as f64;
    let limit = threshold.decorator_density_limit();

    if ratio <= limit {
        return None;
    }

    let severity = if ratio >= limit * 3.0 {
        Severity::High
    } else if ratio > limit * 2.0 {
        Severity::Medium
    } else {
        Severity::Low
    };

    Some(Finding::new(
        next_finding_id("ic", counter),
        FindingKind::ImplicitControl,
        severity,
        0.40,
        file.relative_path.clone(),
        vec![
            format!("decorator_count: {}", decorator_count),
            format!("function_count: {}", fn_count),
            format!("ratio: {:.3}", ratio),
            format!("limit: {:.3}", limit),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn f() -> RustFile {
        RustFile {
            path: PathBuf::from("src/foo.rs"),
            relative_path: PathBuf::from("src/foo.rs"),
            package_name: "krate".to_string(),
            size_bytes: 1000,
            line_count: 50,
        }
    }
    fn s(decorators: usize, fns: usize) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/foo.rs"),
            decorator_count: decorators,
            function_count: fns,
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        // 1 decorator, 2 functions: ratio 0.5, normal limit 0.5 → below
        let r = analyze(&f(), &s(1, 2), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn low_at_limit_plus_one() {
        // 2 decorators, 2 functions: ratio 1.0, normal limit 0.5 → Low
        let r = analyze(&f(), &s(2, 2), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Low);
        assert_eq!(r.confidence, 0.40);
    }

    #[test]
    fn medium_at_2x_limit() {
        // strict limit = 0.33, ratio ≈ 0.67, 0.67 > 0.33*2 = 0.66 → Medium
        let r = analyze(&f(), &s(2, 3), &Threshold::Strict, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
    }

    #[test]
    fn high_at_3x_limit() {
        // strict limit = 0.33, ratio = 1.5, 1.5 >= 0.33*3 = 0.99 → High
        let r = analyze(&f(), &s(3, 2), &Threshold::Strict, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn id_uses_ic_prefix() {
        let r = analyze(&f(), &s(2, 2), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.id.starts_with("ic-"));
    }

    #[test]
    fn evidence_contains_decorator_count_and_ratio() {
        let r = analyze(&f(), &s(3, 2), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.evidence.iter().any(|e| e.starts_with("decorator_count: 3")));
        assert!(r.evidence.iter().any(|e| e.starts_with("function_count: 2")));
        assert!(r.evidence.iter().any(|e| e.starts_with("ratio:")));
        assert!(r.evidence.iter().any(|e| e.starts_with("limit:")));
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f(), &s(3, 2), &Threshold::Normal, &mut c);
        analyze(&f(), &s(3, 2), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
