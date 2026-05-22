//! Naming entropy / mixed-style heuristic.
//!
//! Detects files that use multiple naming conventions (snake_case, camelCase,
//! PascalCase, SCREAMING_SNAKE) in significant proportion. Mixed naming
//! conventions within a file can confuse LLM expectations and reduce
//! the model's ability to reason about the code.

use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};
use std::collections::HashMap;

/// Classify an identifier string into a naming convention.
fn classify_identifier(name: &str) -> &'static str {
    if name.len() < 3 {
        return "other";
    }

    let has_underscore = name.contains('_');
    let all_upper = name.chars().all(|c| !c.is_lowercase());
    let all_lower = name.chars().all(|c| !c.is_uppercase());
    let first_upper = name.chars().next().map_or(false, |c| c.is_uppercase());
    let first_lower = name.chars().next().map_or(false, |c| c.is_lowercase());

    if has_underscore && all_lower {
        return "snake_case";
    }
    if has_underscore && all_upper {
        return "SCREAMING_SNAKE";
    }
    if first_lower && name.chars().any(|c| c.is_uppercase()) {
        return "camelCase";
    }
    if first_upper && name[1..].chars().any(|c| c.is_lowercase()) {
        return "PascalCase";
    }
    "other"
}

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    _threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let identifiers = &structure.all_identifiers;

    // Require a minimum number of identifiers to classify
    if identifiers.len() < 10 {
        return None;
    }

    // Classify each identifier
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let total = identifiers.len();

    for id in identifiers {
        let convention = classify_identifier(id);
        *counts.entry(convention).or_insert(0) += 1;
    }

    // Find conventions that represent >5% of total
    let threshold_count = (total as f64 * 0.05).ceil() as usize;
    let active_conventions: Vec<&str> = counts
        .iter()
        .filter(|(_, &count)| count > threshold_count)
        .map(|(conv, _)| *conv)
        .collect();

    // If 3+ conventions are in significant use, emit a finding
    if active_conventions.len() < 3 {
        return None;
    }

    // Find the dominant convention
    let dominant = counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(conv, _)| *conv)
        .unwrap_or("none");

    *counter += 1;
    Some(Finding {
        id: format!("ne-{:03}", counter),
        kind: FindingKind::NamingEntropy,
        severity: Severity::Low,
        confidence: 0.50,
        path: file.relative_path.clone(),
        summary: String::new(),
        reasons: vec![],
        evidence: vec![
            format!("convention_counts: {}", serde_json::to_string(&counts).unwrap_or_default()),
            format!("dominant_convention: {}", dominant),
            format!("mixed_count: {}", active_conventions.len()),
        ],
        suggested_next_step: String::new(),
        estimated_tokens: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::FileStructure;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn f() -> RustFile {
        RustFile {
            path: PathBuf::from("src/mixed.rs"),
            relative_path: PathBuf::from("src/mixed.rs"),
            package_name: "krate".to_string(),
            size_bytes: 1000,
            line_count: 50,
        }
    }

    fn s(idents: Vec<String>) -> FileStructure {
        FileStructure {
            path: PathBuf::from("src/mixed.rs"),
            all_identifiers: idents.into_iter().collect::<HashSet<_>>(),
            ..Default::default()
        }
    }

    fn make_mixed_idents() -> Vec<String> {
        let mut idents = vec![];
        for i in 0..6 { idents.push(format!("snake_val_{}", i)); }
        for i in 0..6 { idents.push(format!("camelValue{}", i)); }
        for i in 0..6 { idents.push(format!("PascalType{}", i)); }
        idents
    }

    #[test]
    fn no_finding_too_few_identifiers() {
        // Less than 10 identifiers → no finding
        let idents = vec!["x".into(), "y".into(), "foo".into(), "bar".into()];
        assert!(analyze(&f(), &s(idents), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn no_finding_consistent_snake_case() {
        // All snake_case → single convention → no finding
        let idents: Vec<String> = (0..15).map(|i| format!("snake_case_{}", i)).collect();
        assert!(analyze(&f(), &s(idents), &Threshold::Normal, &mut 0).is_none());
    }

    #[test]
    fn finding_mixed_conventions() {
        // 3 conventions each with >5% share → finding
        let idents = make_mixed_idents();
        let r = analyze(&f(), &s(idents), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.kind, FindingKind::NamingEntropy);
        assert_eq!(r.severity, Severity::Low);
        assert!((r.confidence - 0.50).abs() < f32::EPSILON);
    }

    #[test]
    fn id_uses_ne_prefix() {
        let idents = make_mixed_idents();
        let r = analyze(&f(), &s(idents), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.id.starts_with("ne-"));
    }

    #[test]
    fn counter_increments() {
        let idents = make_mixed_idents();
        let mut c = 0;
        analyze(&f(), &s(idents.clone()), &Threshold::Normal, &mut c);
        analyze(&f(), &s(idents), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }

    #[test]
    fn evidence_contains_expected_keys() {
        let idents = make_mixed_idents();
        let r = analyze(&f(), &s(idents), &Threshold::Normal, &mut 0).unwrap();
        assert!(r.evidence.iter().any(|e| e.starts_with("convention_counts:")));
        assert!(r.evidence.iter().any(|e| e.starts_with("dominant_convention:")));
        assert!(r.evidence.iter().any(|e| e.starts_with("mixed_count:")));
    }
}
