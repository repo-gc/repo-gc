use std::collections::HashSet;
use std::path::PathBuf;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{Finding, FindingKind, Severity};

pub fn analyze_orphaned_files(
    files: &[RustFile],
    structures: &[FileStructure],
    counter: &mut usize,
) -> Vec<Finding> {
    // Collect every module path that is referenced anywhere:
    // 1. Fully-qualified mod declarations recorded by the visitor
    // 2. Every prefix of every resolved use path
    let mut referenced: HashSet<String> = HashSet::new();

    for s in structures {
        // mod declarations are already fully qualified (e.g. "my_crate::lib::utils")
        for m in &s.mod_declarations {
            // Insert all prefixes so "my_crate::lib::utils" also marks "my_crate::lib" etc.
            let parts: Vec<&str> = m.split("::").collect();
            for len in 1..=parts.len() {
                referenced.insert(parts[..len].join("::"));
            }
        }

        // Resolve each use path to its absolute module path using the same logic as import_graph,
        // then insert all prefixes.
        let all_uses = s.use_paths.iter().chain(s.pub_use_paths.iter().map(|u| &u.path));
        for use_path in all_uses {
            if let Some(abs) = resolve_to_module_path(use_path, &s.module_path) {
                let parts: Vec<&str> = abs.split("::").collect();
                for len in 1..=parts.len() {
                    referenced.insert(parts[..len].join("::"));
                }
            }
        }
    }

    let mut findings = vec![];
    for file in files {
        let stem = file
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Entry points are never orphaned
        if matches!(stem.as_str(), "lib" | "main" | "mod") {
            continue;
        }

        // Get the package-qualified module path from the corresponding structure
        let module_path = structures
            .iter()
            .find(|s| s.path == file.path)
            .map(|s| s.module_path.clone())
            .unwrap_or_else(|| {
                // Fallback: use package name + stem (avoids stem-only collision)
                format!("{}::{}", file.package_name, stem)
            });

        let is_referenced = referenced.contains(&module_path)
            // Also check all prefixes of module_path in case something imports a child
            || {
                let parts: Vec<&str> = module_path.split("::").collect();
                (1..parts.len()).any(|len| referenced.contains(&parts[..len].join("::")))
            };

        if !is_referenced {
            *counter += 1;
            let severity = if file.line_count >= 1000 {
                Severity::Critical
            } else if file.line_count >= 500 {
                Severity::High
            } else {
                Severity::Medium
            };
            findings.push(Finding {
                id: format!("dw-{:03}", counter),
                kind: FindingKind::DeadWeight,
                severity,
                confidence: 0.6,
                path: file.relative_path.clone(),
                summary: format!(
                    "Orphaned file: {} (module '{}') not referenced by any use or mod declaration",
                    file.relative_path.display(),
                    module_path
                ),
                reasons: vec![format!(
                    "Module path '{}' not found in any use or mod declaration",
                    module_path
                )],
                evidence: vec![
                    format!("module_path: {}", module_path),
                    format!("line_count: {}", file.line_count),
                ],
                suggested_next_step: format!(
                    "Verify {} is still needed; add `mod {}` or delete it",
                    file.relative_path.display(),
                    stem
                ),
                estimated_tokens: Some(crate::heuristics::context_bombs::estimate_tokens(
                    file.size_bytes,
                )),
            });
        }
    }
    findings
}

/// Mirror of import_graph's resolve_to_module_path — kept local to avoid circular deps.
fn resolve_to_module_path(use_path: &str, current_module: &str) -> Option<String> {
    let parts: Vec<&str> = use_path.split("::").collect();
    if parts.is_empty() {
        return None;
    }
    match parts[0] {
        "crate" => {
            let pkg = current_module.split("::").next()?;
            let rest = &parts[1..];
            if rest.is_empty() {
                return Some(pkg.to_string());
            }
            Some(format!("{}::{}", pkg, rest.join("::")))
        }
        "super" => {
            let current_parts: Vec<&str> = current_module.split("::").collect();
            if current_parts.len() < 2 {
                return None;
            }
            let parent_parts = &current_parts[..current_parts.len() - 1];
            let rest = &parts[1..];
            if rest.is_empty() {
                return Some(parent_parts.join("::"));
            }
            Some(format!("{}::{}", parent_parts.join("::"), rest.join("::")))
        }
        "self" => {
            let rest = &parts[1..];
            if rest.is_empty() {
                return Some(current_module.to_string());
            }
            Some(format!("{}::{}", current_module, rest.join("::")))
        }
        _ => Some(use_path.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mkfile(pkg: &str, stem: &str) -> RustFile {
        let name = format!("src/{}.rs", stem);
        RustFile {
            path: PathBuf::from(&name),
            relative_path: PathBuf::from(&name),
            package_name: pkg.to_string(),
            size_bytes: 500,
            line_count: 20,
        }
    }

    fn mks(path: &str, pkg: &str, module: &str, use_paths: Vec<&str>, mods: Vec<String>) -> FileStructure {
        FileStructure {
            path: PathBuf::from(path),
            relative_path: PathBuf::from(path),
            package_name: pkg.to_string(),
            module_path: format!("{}::{}", pkg, module),
            use_paths: use_paths.into_iter().map(String::from).collect(),
            mod_declarations: mods,
            pub_use_paths: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn detects_orphaned_file() {
        let files = vec![mkfile("krate", "orphan"), mkfile("krate", "lib")];
        let structures = vec![mks("src/lib.rs", "krate", "lib", vec![], vec![])];
        let r = analyze_orphaned_files(&files, &structures, &mut 0);
        assert_eq!(r.len(), 1);
        assert!(r[0].path.to_string_lossy().contains("orphan"));
    }

    #[test]
    fn fully_qualified_mod_declaration_prevents_orphan() {
        let files = vec![mkfile("krate", "utils"), mkfile("krate", "lib")];
        // lib declares `mod utils` → visitor records "krate::lib::utils"
        let structures = vec![mks(
            "src/lib.rs",
            "krate",
            "lib",
            vec![],
            vec!["krate::lib::utils".to_string()],
        )];
        assert!(analyze_orphaned_files(&files, &structures, &mut 0).is_empty());
    }

    #[test]
    fn crate_prefix_use_prevents_orphan() {
        let files = vec![mkfile("krate", "utils"), mkfile("krate", "lib")];
        let structures = vec![mks("src/lib.rs", "krate", "lib", vec!["crate::utils::Foo"], vec![])];
        assert!(analyze_orphaned_files(&files, &structures, &mut 0).is_empty());
    }

    #[test]
    fn nested_module_use_prevents_orphan() {
        let f = RustFile {
            path: PathBuf::from("src/discovery/files.rs"),
            relative_path: PathBuf::from("src/discovery/files.rs"),
            package_name: "krate".to_string(),
            size_bytes: 500,
            line_count: 20,
        };
        let structures = vec![
            mks(
                "src/lib.rs",
                "krate",
                "lib",
                vec!["crate::discovery::files::RustFile"],
                vec![],
            ),
            mks("src/discovery/files.rs", "krate", "discovery::files", vec![], vec![]),
        ];
        assert!(analyze_orphaned_files(&[f], &structures, &mut 0).is_empty());
    }

    #[test]
    fn entry_points_never_orphaned() {
        let files = vec![mkfile("krate", "main"), mkfile("krate", "lib"), mkfile("krate", "mod")];
        assert!(analyze_orphaned_files(&files, &[], &mut 0).is_empty());
    }

    #[test]
    fn same_stem_different_packages_no_collision() {
        // pkg_a/src/utils.rs and pkg_b/src/utils.rs are different modules
        let file_a = RustFile {
            path: PathBuf::from("pkg_a/src/utils.rs"),
            relative_path: PathBuf::from("pkg_a/src/utils.rs"),
            package_name: "pkg_a".to_string(),
            size_bytes: 500,
            line_count: 20,
        };
        let file_b = RustFile {
            path: PathBuf::from("pkg_b/src/utils.rs"),
            relative_path: PathBuf::from("pkg_b/src/utils.rs"),
            package_name: "pkg_b".to_string(),
            size_bytes: 500,
            line_count: 20,
        };
        // Only pkg_a references its utils
        let structures = vec![mks(
            "pkg_a/src/lib.rs",
            "pkg_a",
            "lib",
            vec!["crate::utils::Foo"],
            vec![],
        )];
        let orphans = analyze_orphaned_files(&[file_a, file_b], &structures, &mut 0);
        // pkg_b::utils is orphaned, pkg_a::utils is not
        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].path.to_string_lossy().contains("pkg_b"));
    }
}
