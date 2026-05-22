//! Import diversity / God file heuristic.
//!
//! Detects files that import from too many unrelated domains — a "god file"
//! that does too much. Backed by the Program Decomposition paper
//! (arXiv 2401.12412): reducing cross-file dependencies shrinks context to
//! ~5% of window.
//!
//! Logic: count distinct first-segment domains across all use paths and
//! pub re-exports. If count > limit → finding.
//! Severity: >= limit*3 → High, >= limit*2 → Medium, >= limit → Low.

use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};

/// Extract the domain from a Rust use path.
///
/// - `std::collections::HashMap` → Some("std")
/// - `serde::Serialize` → Some("serde")
/// - `crate::foo::bar` → Some("foo")        (skip `crate`)
/// - `super::baz` → None                    (relative)
/// - `self::qux` → None                     (relative)
/// - `Bar as baz` → Some("Bar")             (rename, strip alias)
fn extract_domain(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split("::").collect();
    let first = parts.first()?;

    match *first {
        "self" | "super" => None,
        "crate" => {
            let second = parts.get(1)?;
            Some(clean_domain(second))
        }
        other => Some(clean_domain(other)),
    }
}

/// Strip " as alias" suffix from renamed imports.
fn clean_domain(s: &str) -> String {
    s.split(" as ").next().unwrap_or(s).trim().to_string()
}

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let limit = threshold.import_domain_limit();

    let mut domains = std::collections::BTreeSet::new();

    // Collect domains from private use paths
    for path in &structure.use_paths {
        if let Some(domain) = extract_domain(path) {
            domains.insert(domain);
        }
    }

    // Collect domains from public re-export paths
    for entry in &structure.pub_use_paths {
        if let Some(domain) = extract_domain(&entry.path) {
            domains.insert(domain);
        }
    }

    let count = domains.len();

    if count <= limit {
        return None;
    }

    let severity = if count >= limit * 3 {
        Severity::High
    } else if count >= limit * 2 {
        Severity::Medium
    } else {
        Severity::Low
    };

    // Top 8 domains
    let top_domains: Vec<&str> = domains.iter().map(String::as_str).take(8).collect();
    let domains_str = top_domains.join(", ");

    *counter += 1;
    Some(Finding {
        id: format!("id-{:03}", counter),
        kind: FindingKind::ImportDiversity,
        severity,
        confidence: 0.70,
        path: file.relative_path.clone(),
        summary: String::new(),
        reasons: vec![],
        evidence: vec![
            format!("domain_count: {}", count),
            format!("domains: {}", domains_str),
            format!("limit: {}", limit),
        ],
        suggested_next_step: String::new(),
        estimated_tokens: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::extractor::PubUseEntry;
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

    fn s(use_paths: Vec<&str>, pub_use_paths: Vec<&str>) -> FileStructure {
        FileStructure {
            use_paths: use_paths.into_iter().map(String::from).collect(),
            pub_use_paths: pub_use_paths
                .into_iter()
                .map(|p| PubUseEntry {
                    path: p.to_string(),
                    item_count: 1,
                    is_wildcard: false,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        // Normal limit=10, only 3 domains
        let paths = vec!["std::collections::HashMap", "serde::Serialize", "tokio::runtime"];
        let r = analyze(&f("src/test.rs"), &s(paths, vec![]), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn no_finding_at_exact_limit() {
        // Normal limit=10, exactly 10 domains
        let paths: Vec<String> = (1..=10).map(|i| format!("crate_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
    }

    #[test]
    fn finding_above_limit() {
        // Normal limit=10, 12 domains → finding
        let paths: Vec<String> = (1..=12).map(|i| format!("crate_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        assert!(r.is_some());
        let f = r.unwrap();
        assert_eq!(f.kind, FindingKind::ImportDiversity);
        assert!((f.confidence - 0.70).abs() < 1e-6);
    }

    #[test]
    fn severity_low_at_limit() {
        // Normal limit=10, 11 domains → Low (just above limit)
        let paths: Vec<String> = (1..=11).map(|i| format!("crate_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        assert_eq!(r.unwrap().severity, Severity::Low);
    }

    #[test]
    fn severity_medium_at_double_limit() {
        // Normal limit=10, 20 domains → Medium (>= 2*limit)
        let paths: Vec<String> = (1..=20).map(|i| format!("crate_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        assert_eq!(r.unwrap().severity, Severity::Medium);
    }

    #[test]
    fn severity_high_at_triple_limit() {
        // Normal limit=10, 30 domains → High (>= 3*limit)
        let paths: Vec<String> = (1..=30).map(|i| format!("crate_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        assert_eq!(r.unwrap().severity, Severity::High);
    }

    #[test]
    fn skips_relative_imports() {
        // self:: and super:: should be skipped
        let paths = vec!["self::inner::helper", "super::parent::util", "std::collections"];
        let r = analyze(&f("src/test.rs"), &s(paths, vec![]), &Threshold::Normal, &mut 0);
        // Only "std" domain counted
        assert!(r.is_none());
    }

    #[test]
    fn handles_crate_prefix() {
        // crate::foo::bar → domain "foo"
        let paths = vec!["crate::foo::bar", "crate::baz::qux", "std::collections"];
        let r = analyze(&f("src/test.rs"), &s(paths, vec![]), &Threshold::Normal, &mut 0);
        assert!(r.is_none());
        // domains should be: foo, baz, std → 3 domains
    }

    #[test]
    fn combines_use_and_pub_use() {
        // Private + public re-exports both contribute
        let use_paths = vec!["std::collections::HashMap", "serde::Serialize"];
        let pub_use_paths = vec!["tokio::runtime::Runtime", "reqwest::Client"];
        let r = analyze(&f("src/test.rs"), &s(use_paths, pub_use_paths), &Threshold::Strict, &mut 0);
        // Strict limit=8, 4 domains → no finding
        assert!(r.is_none());

        // 9 domains → finding
        let many_use: Vec<String> = (1..=5).map(|i| format!("dep_{}::foo", i)).collect();
        let many_pub: Vec<String> = (6..=10).map(|i| format!("dep_{}::bar", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(many_use.iter().map(String::as_str).collect(), many_pub.iter().map(String::as_str).collect()), &Threshold::Strict, &mut 0);
        assert!(r.is_some());
        assert_eq!(r.unwrap().severity, Severity::Low);
    }

    #[test]
    fn handles_renamed_imports() {
        // renamed: `use Bar as baz` → domain "Bar"
        let paths = vec!["Bar as baz", "Baz as qux", "std::collections"];
        let r = analyze(&f("src/test.rs"), &s(paths, vec![]), &Threshold::Normal, &mut 0);
        assert!(r.is_none()); // 3 domains, below limit=10
    }

    #[test]
    fn counter_increments() {
        let many: Vec<String> = (1..=15).map(|i| format!("dep_{}::foo", i)).collect();
        let mut c = 0;
        analyze(&f("src/test.rs"), &s(many.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut c);
        analyze(&f("src/test.rs"), &s(many.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }

    #[test]
    fn evidence_contains_domain_count_and_limit() {
        let paths: Vec<String> = (1..=12).map(|i| format!("dep_{}::foo", i)).collect();
        let r = analyze(&f("src/test.rs"), &s(paths.iter().map(String::as_str).collect(), vec![]), &Threshold::Normal, &mut 0);
        let f = r.unwrap();
        assert!(f.evidence.iter().any(|e| e.contains("domain_count: 12")));
        assert!(f.evidence.iter().any(|e| e.contains("limit: 10")));
        assert!(f.evidence.iter().any(|e| e.starts_with("domains: ")));
    }
}
