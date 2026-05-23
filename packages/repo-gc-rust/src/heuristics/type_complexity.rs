//! Type complexity heuristic — detects deeply nested generic/union types.
//!
//! Counts maximum nesting depth of type expressions (e.g.
//! `Option<HashMap<K, Vec<T>>>` = depth 3). Highest FP risk — only fires
//! at extreme depths (>5).

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
    let limit = threshold.type_depth_limit();
    if structure.max_type_depth <= limit {
        return None;
    }

    let depth = structure.max_type_depth;
    let severity = severity_scale_offset(depth, limit, 2, 3);

    Some(Finding::new(
        next_finding_id("tc", counter),
        FindingKind::TypeComplexity,
        severity,
        0.50,
        file.relative_path.clone(),
        vec![
            format!("max_type_depth: {}", depth),
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
            path: PathBuf::from("src/foo.rs"),
            relative_path: PathBuf::from("src/foo.rs"),
            package_name: "krate".to_string(),
            size_bytes: 1000,
            line_count: 50,
        }
    }
    fn s(depth: usize) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/foo.rs"),
            max_type_depth: depth,
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        assert!(analyze(&f(), &s(3), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn medium_at_limit_plus_one() {
        let r = analyze(&f(), &s(5), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Medium);
        assert_eq!(r.confidence, 0.50);
    }

    #[test]
    fn high_at_limit_plus_2() {
        let r = analyze(&f(), &s(6), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
    }

    #[test]
    fn critical_at_limit_plus_3() {
        let r = analyze(&f(), &s(7), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::Critical);
    }

    #[test]
    fn id_uses_tc_prefix() {
        let r = analyze(&f(), &s(5), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.id.starts_with("tc-"));
    }

    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f(), &s(5), &Threshold::Normal, &mut c);
        analyze(&f(), &s(5), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }

    #[test]
    fn evidence_contains_depth_and_limit() {
        let r = analyze(&f(), &s(5), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.evidence.iter().any(|e| e.starts_with("max_type_depth: 5")));
        assert!(r.evidence.iter().any(|e| e.starts_with("limit: 4")));
    }
}
