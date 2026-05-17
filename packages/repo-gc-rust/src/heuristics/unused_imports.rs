use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};

/// Minimum unused imports before emitting a finding (reduces noise from macros/derives).
const MIN_UNUSED: usize = 2;

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    _threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let unused: Vec<&String> = structure
        .use_leaf_names
        .iter()
        .filter(|name| !structure.all_identifiers.contains(*name))
        .collect();

    if unused.len() < MIN_UNUSED {
        return None;
    }

    let severity = if unused.len() >= 5 {
        Severity::Medium
    } else {
        Severity::Low
    };

    *counter += 1;
    let preview: Vec<&str> = unused.iter().take(5).map(|s| s.as_str()).collect();
    Some(Finding {
        id: format!("ui-{:03}", counter),
        kind: FindingKind::UnusedImport,
        severity,
        // Low confidence: derive macros and proc-macros may reference names invisibly
        confidence: 0.65,
        path: file.relative_path.clone(),
        summary: format!(
            "Zombie imports — {} unused names inflate token usage in every context window: {}",
            unused.len(),
            preview.join(", ")
        ),
        reasons: vec![format!(
            "{} imported names not referenced in file body; macros/derives may cause false positives",
            unused.len()
        )],
        evidence: unused.iter().map(|n| format!("imported but unreferenced: {n}")).collect(),
        suggested_next_step: "Remove zombie imports to reduce token waste, or verify they are needed by derive macros"
            .into(),
        estimated_tokens: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Threshold;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn mkfile(stem: &str) -> RustFile {
        RustFile {
            path: PathBuf::from(format!("src/{stem}.rs")),
            relative_path: PathBuf::from(format!("src/{stem}.rs")),
            package_name: "krate".into(),
            size_bytes: 200,
            line_count: 10,
        }
    }

    fn mks(leaf_names: Vec<&str>, used_idents: Vec<&str>) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/foo.rs"),
            relative_path: PathBuf::from("src/foo.rs"),
            package_name: "krate".into(),
            module_path: "krate::foo".into(),
            use_leaf_names: leaf_names.into_iter().map(String::from).collect(),
            all_identifiers: used_idents.into_iter().map(String::from).collect::<HashSet<_>>(),
            pub_use_paths: vec![],
            use_paths: vec![],
            mod_declarations: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_when_all_used() {
        let f = mkfile("foo");
        let s = mks(vec!["HashMap", "Arc"], vec!["HashMap", "Arc", "fn_body"]);
        assert!(analyze(&f, &s, &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn no_finding_below_min_threshold() {
        let f = mkfile("foo");
        let s = mks(vec!["HashMap"], vec!["something_else"]);
        assert!(analyze(&f, &s, &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn finding_when_multiple_unused() {
        let f = mkfile("foo");
        let s = mks(vec!["Foo", "Bar", "Baz"], vec!["something_else"]);
        let finding = analyze(&f, &s, &Threshold::Normal, &mut 0);
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().kind, FindingKind::UnusedImport);
    }

    #[test]
    fn medium_severity_at_five_or_more() {
        let f = mkfile("foo");
        let s = mks(vec!["A", "B", "C", "D", "E"], vec![]);
        let finding = analyze(&f, &s, &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(finding.severity, Severity::Medium);
    }
}
