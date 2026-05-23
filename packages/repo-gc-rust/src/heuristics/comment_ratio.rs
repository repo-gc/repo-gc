//! Comment ratio heuristic.
//!
//! Detects files with too few comments (LLMs can't infer intent) or too many
//! (token waste). Uses research-backed thresholds: min 0.03 (3%), max 0.20 (20%)
//! for strict mode.

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
    let line_count = file.line_count.max(1);
    let ratio = structure.comment_line_count as f64 / line_count as f64;

    if ratio < threshold.comment_ratio_min() {
        return Some(Finding::new(
            next_finding_id("cr", counter),
            FindingKind::CommentRatio,
            Severity::Medium,
            0.65,
            file.relative_path.clone(),
            vec![
                format!("comment_lines: {}", structure.comment_line_count),
                format!("total_lines: {}", file.line_count),
                format!("ratio: {:.4}", ratio),
                format!("direction: sparse"),
            ],
        ));
    }

    if ratio > threshold.comment_ratio_max() {
        return Some(Finding::new(
            next_finding_id("cr", counter),
            FindingKind::CommentRatio,
            Severity::Low,
            0.65,
            file.relative_path.clone(),
            vec![
                format!("comment_lines: {}", structure.comment_line_count),
                format!("total_lines: {}", file.line_count),
                format!("ratio: {:.4}", ratio),
                format!("direction: verbose"),
            ],
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn f(name: &str, line_count: usize) -> RustFile {
        RustFile {
            path: PathBuf::from(name),
            relative_path: PathBuf::from(name),
            package_name: "krate".to_string(),
            size_bytes: 100,
            line_count,
        }
    }

    fn s(comment_lines: usize) -> FileStructure {
        FileStructure {
            comment_line_count: comment_lines,
            path: PathBuf::from("src/lib.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_when_ratio_in_normal_range() {
        // Normal thresholds: min 0.03, max 0.30 → 50/1000 = 0.05 → OK
        assert!(analyze(&f("src/lib.rs", 1000), &s(50), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn sparse_finding_when_below_min() {
        // Normal min 0.03 → 1/100 = 0.01 → sparse
        let r = analyze(&f("src/lib.rs", 100), &s(1), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::CommentRatio);
        assert_eq!(r.severity, Severity::Medium);
        assert!(r.evidence.iter().any(|e| e.contains("direction: sparse")));
    }

    #[test]
    fn verbose_finding_when_above_max() {
        // Normal max 0.30 → 400/1000 = 0.40 → verbose
        let r = analyze(&f("src/lib.rs", 1000), &s(400), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::CommentRatio);
        assert_eq!(r.severity, Severity::Low);
        assert!(r.evidence.iter().any(|e| e.contains("direction: verbose")));
    }

    #[test]
    fn no_finding_at_exact_min() {
        let r = analyze(&f("src/lib.rs", 100), &s(3), &Threshold::Strict, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_at_exact_max() {
        // Strict max 0.20 → 20/100 = 0.20 → equal, no finding
        let r = analyze(&f("src/lib.rs", 100), &s(20), &Threshold::Strict, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn relaxed_mode_has_wider_range() {
        // Relaxed min 0.01 → 1/200 = 0.005 → sparse
        let r = analyze(&f("src/lib.rs", 200), &s(1), &Threshold::Relaxed, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::CommentRatio);
        assert!(r.evidence.iter().any(|e| e.contains("direction: sparse")));
    }

    #[test]
    fn evidence_contains_ratio_and_counts() {
        let r = analyze(&f("src/lib.rs", 200), &s(1), &Threshold::Strict, &mut 0).unwrap();
        assert!(r.evidence.iter().any(|e| e.starts_with("comment_lines: 1")));
        assert!(r.evidence.iter().any(|e| e.starts_with("total_lines: 200")));
        assert!(r.evidence.iter().any(|e| e.starts_with("ratio:")));
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f("src/lib.rs", 200), &s(1), &Threshold::Strict, &mut c);
        analyze(&f("src/lib.rs", 200), &s(1), &Threshold::Strict, &mut c);
        assert_eq!(c, 2);
    }
}
