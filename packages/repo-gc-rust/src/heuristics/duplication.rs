use std::collections::HashMap;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};
use syn::fold::Fold;

/// Minimum normalized token length to consider a body worth comparing.
const MIN_BODY_LEN: usize = 40;

pub fn analyze_duplicates(structures: &[FileStructure], counter: &mut usize) -> Vec<Finding> {
    let mut body_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for s in structures {
        for (fn_name, body) in &s.function_bodies {
            if is_test_only_name(fn_name) {
                continue;
            }
            // Check raw body length before identifier normalization,
            // since normalization collapses all idents to `_`.
            let raw_key = normalize(body);
            if raw_key.len() < MIN_BODY_LEN {
                continue;
            }
            if is_trivial_body(body) {
                continue;
            }
            let key = normalize_identifiers(body);
            body_map
                .entry(key)
                .or_default()
                .push((s.relative_path.display().to_string(), fn_name.clone()));
        }
    }

    let mut findings = vec![];
    for (_body, locations) in &body_map {
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
        let files_list: Vec<String> = locations
            .iter()
            .take(5)
            .map(|(path, _)| path.clone())
            .collect();

        let primary_path = std::path::PathBuf::from(&locations[0].0);
        findings.push(Finding {
            id: format!("dup-{:03}", counter),
            kind: FindingKind::CodeDuplication,
            severity,
            confidence: 0.85,
            path: primary_path,
            summary: String::new(),
            reasons: vec![],
            evidence: vec![
                format!("file_count: {}", file_count),
                format!("fn_name: {}", locations[0].1),
                format!("files: {}", files_list.join(", ")),
            ],
            suggested_next_step: String::new(),
            estimated_tokens: None,
        });
    }

    findings.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());
    findings
}

/// Skip functions whose body is a single statement — these are delegations
/// or structural templates, not meaningful duplicated logic.
fn is_trivial_body(body: &str) -> bool {
    let block: syn::Block = match syn::parse_str(body) {
        Ok(b) => b,
        Err(_) => return false,
    };
    block.stmts.len() <= 1
}

/// Skip functions named like test-only or dead-code-only entry points.
/// These are intentionally identical across feature files and not
/// meaningful duplication targets.
fn is_test_only_name(name: &str) -> bool {
    name.contains("for_test")
}

/// Replace all identifiers with a placeholder so that bodies differing only
/// in variable names produce the same normalized key (Type 2 clone detection).
///
/// Example: `{ let x = a + b; x }` and `{ let y = c + d; y }` both become
/// `{let_=_+_;_}` after normalization.
fn normalize_identifiers(body: &str) -> String {
    let block: syn::Block = match syn::parse_str(body) {
        Ok(b) => b,
        Err(_) => return normalize(body),
    };
    let mut normalizer = IdentNormalizer;
    let normalized = normalizer.fold_block(block);
    normalize(&quote::quote!(#normalized).to_string())
}

struct IdentNormalizer;

impl Fold for IdentNormalizer {
    fn fold_ident(&mut self, _: syn::Ident) -> syn::Ident {
        syn::Ident::new("_", proc_macro2::Span::call_site())
    }
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

    // ------------------------------------------------------------------
    // New tests for the improvements
    // ------------------------------------------------------------------

    #[test]
    fn trivial_single_stmt_body_is_filtered() {
        // Single-statement delegation: `{ encode_impl(a, b, c) }`
        let body = "{ encode_impl ( pieces , p_r , p_f , lo , hi , shared_pfx ) }";
        let s = vec![
            mks("src/a.rs", "compute", body),
            mks("src/b.rs", "compute", body),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn test_only_name_is_filtered() {
        let body = "{ let x = a + b + c + d + e + f + g + h + i + j ; x }";
        let s = vec![
            mks("src/a.rs", "encode_for_test", body),
            mks("src/b.rs", "encode_for_test", body),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn type2_clone_renamed_variables_detected() {
        // Same structure, different variable names — must be long enough for MIN_BODY_LEN
        let body1 = "{ let x = a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p + q ; x }";
        let body2 = "{ let y = x1 + x2 + x3 + x4 + x5 + x6 + x7 + x8 + x9 + a + b + c + d + e + f + g + h ; y }";
        let s = vec![
            mks("src/a.rs", "compute", body1),
            mks("src/b.rs", "compute", body2),
        ];
        let findings = analyze_duplicates(&s, &mut 0);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn type2_clone_different_return_var_detected() {
        let body1 = "{ let total = price + tax + shipping + handling + duty + fee ; total }";
        let body2 = "{ let sum = cost + vat + delivery + surcharge + tariff + charge ; sum }";
        let s = vec![
            mks("src/a.rs", "calc", body1),
            mks("src/b.rs", "calc", body2),
        ];
        let findings = analyze_duplicates(&s, &mut 0);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn different_operators_not_flagged() {
        // Same structure but different operators — not a clone
        let body1 = "{ let x = a + b + c + d + e + f + g + h ; x }";
        let body2 = "{ let y = p * q * r * s * t * u * v * w ; y }";
        let s = vec![
            mks("src/a.rs", "add", body1),
            mks("src/b.rs", "mul", body2),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

    #[test]
    fn multistmt_body_not_filtered_as_trivial() {
        // Two statements → not trivial, should be compared
        let body = "{ let x = a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p ; x + 1 }";
        let s = vec![
            mks("src/a.rs", "compute", body),
            mks("src/b.rs", "compute", body),
        ];
        let findings = analyze_duplicates(&s, &mut 0);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn empty_body_is_trivial() {
        let s = vec![
            mks("src/a.rs", "empty", "{ }"),
            mks("src/b.rs", "empty", "{ }"),
        ];
        assert!(analyze_duplicates(&s, &mut 0).is_empty());
    }

}
