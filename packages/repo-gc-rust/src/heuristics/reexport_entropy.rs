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
    let limit = threshold.reexport_limit();
    let pub_use_count = structure.pub_use_paths.len();
    let total_items: usize = structure.pub_use_paths.iter().map(|u| u.item_count).sum();
    let has_wildcard = structure.pub_use_paths.iter().any(|u| u.is_wildcard);

    if pub_use_count < 3 && total_items < limit {
        return None;
    }

    let severity = if total_items >= limit * 3 || has_wildcard {
        Severity::High
    } else if total_items >= limit {
        Severity::Medium
    } else {
        Severity::Low
    };

    let mut reasons = vec![
        format!("{} pub use declarations", pub_use_count),
        format!("{} total re-exported symbols", total_items),
    ];
    if has_wildcard {
        reasons.push("Contains wildcard re-exports (pub use foo::*)".to_string());
    }

    *counter += 1;
    Some(Finding {
        id: format!("re-{:03}", counter),
        kind: FindingKind::ReexportEntropy,
        severity,
        confidence: 0.8,
        path: file.relative_path.clone(),
        summary: format!(
            "Re-export chain — {} symbols via pub use, agents traverse multiple files to resolve each import",
            total_items
        ),
        reasons,
        evidence: vec![
            format!("pub_use_count: {}", pub_use_count),
            format!("total_reexported_items: {}", total_items),
            format!("has_wildcard: {}", has_wildcard),
        ],
        suggested_next_step: format!(
            "Flatten the re-export chain in {} — barrel files degrade LLM path resolution",
            file.relative_path.display()
        ),
        estimated_tokens: None,
    })
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
}
