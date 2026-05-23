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
    let limit = threshold.reexport_limit();
    let pub_use_count = structure.pub_use_paths.len();
    let total_items: usize = structure.pub_use_paths.iter().map(|u| u.item_count).sum();
    let has_wildcard = structure.pub_use_paths.iter().any(|u| u.is_wildcard);

    if pub_use_count < 3 && total_items < limit && !has_wildcard {
        return None;
    }

    let severity = if total_items >= limit * 3 || has_wildcard {
        Severity::High
    } else if total_items >= limit {
        Severity::Medium
    } else {
        Severity::Low
    };

    Some(Finding::new(
        next_finding_id("re", counter),
        FindingKind::ReexportEntropy,
        severity,
        0.8,
        file.relative_path.clone(),
        vec![
            format!("reexport_count: {}", pub_use_count),
            format!("total_items: {}", total_items),
            format!("has_wildcard: {}", has_wildcard),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::extractor::PubUseEntry;
    use std::path::PathBuf;

    fn mkfile() -> RustFile {
        RustFile {
            path: PathBuf::from("src/api/mod.rs"),
            relative_path: PathBuf::from("src/api/mod.rs"),
            package_name: "krate".to_string(),
            size_bytes: 2000,
            line_count: 50,
        }
    }
    fn mks(uses: Vec<(&str, usize, bool)>) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/api/mod.rs"),
            pub_use_paths: uses
                .into_iter()
                .map(|(p, c, w)| PubUseEntry {
                    path: p.into(),
                    item_count: c,
                    is_wildcard: w,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_for_few() {
        assert!(
            analyze(&mkfile(), &mks(vec![("types", 2, false)]), &Threshold::Normal, &mut 0)
                .is_none()
        );
    }
    #[test]
    fn flags_barrel() {
        let f = analyze(
            &mkfile(),
            &mks(vec![
                ("types", 5, false),
                ("handlers", 8, false),
                ("models", 7, false),
            ]),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert_eq!(f.kind, FindingKind::ReexportEntropy);
    }
    #[test]
    fn wildcard_raises_to_high() {
        let f = analyze(
            &mkfile(),
            &mks(vec![("a", 1, false), ("b", 1, false), ("c", 1, true)]),
            &Threshold::Normal,
            &mut 0,
        )
        .unwrap();
        assert_eq!(f.severity, Severity::High);
    }

    #[test]
    fn single_wildcard_detected() {
        let f = analyze(
            &mkfile(),
            &mks(vec![("a", 1, true)]),
            &Threshold::Normal,
            &mut 0,
        );
        assert!(f.is_some(), "single wildcard pub use should be detected");
        assert_eq!(f.unwrap().severity, Severity::High);
    }
}
