use std::collections::HashMap;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};

/// Minimum normalized token length to consider a body worth comparing.
/// Filters out trivial one-liners like `{}`, `{ self.x }`, `{ unimplemented!() }`.
const MIN_BODY_LEN: usize = 40;

pub fn analyze_duplicates(structures: &[FileStructure], counter: &mut usize) -> Vec<Finding> {
    // Map normalized_body → list of (file_path_str, fn_name)
    let mut body_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for s in structures {
        for (fn_name, body) in &s.function_bodies {
            let key = normalize(body);
            if key.len() < MIN_BODY_LEN {
                continue;
            }
            body_map
                .entry(key)
                .or_default()
                .push((s.path.display().to_string(), fn_name.clone()));
        }
    }

    let mut findings = vec![];
    for (_body, locations) in &body_map {
        // Only flag if the same body appears in 2+ distinct files (not just overloads in one file)
        let unique_files: std::collections::HashSet<&str> =
            locations.iter().map(|(p, _)| p.as_str()).collect();
        if unique_files.len() < 2 {
            continue;
        }

        let file_count = unique_files.len();
        let severity = if file_count >= 5 {
            Severity::High
        } else if file_count >= 3 {
            Severity::Medium
        } else {
            Severity::Low
        };

        *counter += 1;
        let evidence: Vec<String> = locations
            .iter()
            .take(4)
            .map(|(path, name)| format!("{path} :: {name}"))
            .collect();

        let primary_path = std::path::PathBuf::from(&locations[0].0);
        findings.push(Finding {
            id: format!("dup-{:03}", counter),
            kind: FindingKind::CodeDuplication,
            severity,
            confidence: 0.85,
            path: primary_path,
            summary: format!(
                "Function body duplicated across {file_count} files (fn `{}`)",
                locations[0].1
            ),
            reasons: vec![format!(
                "Identical function body found in {file_count} different files"
            )],
            evidence,
            suggested_next_step: "Extract duplicated logic into a shared utility function or trait"
                .into(),
            estimated_tokens: None,
        });
    }

    findings.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());
    findings
}

/// Strip all whitespace from a token string for stable comparison.
fn normalize(body: &str) -> String {
    body.split_whitespace().collect::<Vec<_>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mks(path: &str, fn_name: &str, body: &str) -> FileStructure {
        FileStructure {
            path: PathBuf::from(path),
            relative_path: PathBuf::from(path),
            package_name: "krate".into(),
            module_path: format!("krate::{}", path.replace('/', "::")),
            function_bodies: vec![(fn_name.into(), body.into())],
            ..Default::default()
        }
    }

    #[test]
    fn no_findings_for_unique_bodies() {
        let s = vec![
            mks("src/a.rs", "foo", "{ let x = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 ; x }"),
            mks("src/b.rs", "bar", "{ let y = 1 * 2 * 3 * 4 * 5 * 6 * 7 * 8 * 9 * 10 ; y }"),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn detects_duplicate_across_files() {
        let body = "{ let x = some_function ( ) + another_function ( ) ; x * 2 + offset }";
        let s = vec![
            mks("src/a.rs", "compute", body),
            mks("src/b.rs", "compute", body),
        ];
        let findings = analyze_duplicates(&s, &mut 0);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::CodeDuplication);
    }

    #[test]
    fn same_body_in_one_file_not_flagged() {
        let body = "{ let x = some_function ( ) + another_function ( ) ; x * 2 + offset }";
        let s = vec![
            mks("src/a.rs", "foo", body),
            mks("src/a.rs", "bar", body),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn short_bodies_ignored() {
        let s = vec![
            mks("src/a.rs", "new", "{ Self { } }"),
            mks("src/b.rs", "new", "{ Self { } }"),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn severity_scales_with_file_count() {
        let body = "{ let result = alpha ( ) + beta ( ) + gamma ( ) + delta ( ) + epsilon ( ) }";
        let s: Vec<_> = (0..5)
            .map(|i| mks(&format!("src/file{i}.rs"), "work", body))
            .collect();
        let findings = analyze_duplicates(&s, &mut 0);
        assert_eq!(findings[0].severity, Severity::High);
    }
}
