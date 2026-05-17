use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::parsing::FileStructure;

#[derive(Debug, Default)]
pub struct ImportGraph {
    pub fan_in: HashMap<PathBuf, usize>,
    pub fan_out: HashMap<PathBuf, usize>,
}

/// Resolve a use path to an absolute module path given the current file's module path.
///
/// - `crate::foo::bar` → strip `crate::`, resolve `foo::bar` from workspace root
/// - `super::foo::bar` → walk up one segment from current module, prepend parent
/// - `self::foo::bar`  → resolve from current module
/// - `foo::bar`        → try as-is (may be an external crate or local module)
fn resolve_to_module_path(use_path: &str, current_module: &str) -> Option<String> {
    let parts: Vec<&str> = use_path.split("::").collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "crate" => {
            // crate:: paths are absolute within the package.
            // The package name is the first segment of current_module.
            let pkg = current_module.split("::").next()?;
            let rest = &parts[1..];
            if rest.is_empty() {
                return Some(pkg.to_string());
            }
            Some(format!("{}::{}", pkg, rest.join("::")))
        }
        "super" => {
            // super:: walks up one segment from the current module's parent.
            let current_parts: Vec<&str> = current_module.split("::").collect();
            // Drop the last segment to get the parent module
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
            // self:: is relative to the current module
            let rest = &parts[1..];
            if rest.is_empty() {
                return Some(current_module.to_string());
            }
            Some(format!("{}::{}", current_module, rest.join("::")))
        }
        _ => {
            // Bare path — could be external crate or sibling module.
            // Return as-is; longest-prefix matching will handle it.
            Some(use_path.to_string())
        }
    }
}

/// Find the file that provides the given absolute module path using longest-prefix matching.
fn resolve_import<'a>(
    absolute_module_path: &str,
    module_to_path: &'a HashMap<String, PathBuf>,
) -> Option<&'a PathBuf> {
    let parts: Vec<&str> = absolute_module_path.split("::").collect();
    for len in (1..=parts.len()).rev() {
        let prefix = parts[..len].join("::");
        if let Some(p) = module_to_path.get(&prefix) {
            return Some(p);
        }
    }
    None
}

impl ImportGraph {
    pub fn build(structures: &[FileStructure], _root: &Path) -> Self {
        let mut fan_in: HashMap<PathBuf, usize> = HashMap::new();
        let mut fan_out: HashMap<PathBuf, usize> = HashMap::new();

        for s in structures {
            fan_in.entry(s.path.clone()).or_insert(0);
            fan_out.entry(s.path.clone()).or_insert(0);
        }

        // Build package-qualified module_path → absolute_path map
        let module_to_path: HashMap<String, PathBuf> = structures
            .iter()
            .filter(|s| !s.module_path.is_empty())
            .map(|s| (s.module_path.clone(), s.path.clone()))
            .collect();

        for s in structures {
            let mut out_count = 0usize;
            let all_uses = s
                .use_paths
                .iter()
                .chain(s.pub_use_paths.iter().map(|u| &u.path));

            for use_path in all_uses {
                // Resolve to an absolute (package-qualified) module path
                let abs = match resolve_to_module_path(use_path, &s.module_path) {
                    Some(a) => a,
                    None => continue,
                };

                if let Some(target) = resolve_import(&abs, &module_to_path) {
                    if target != &s.path {
                        *fan_in.entry(target.clone()).or_insert(0) += 1;
                        out_count += 1;
                    }
                }
            }
            *fan_out.entry(s.path.clone()).or_insert(0) = out_count;
        }

        ImportGraph { fan_in, fan_out }
    }

    pub fn get_fan_in(&self, path: &Path) -> usize {
        self.fan_in.get(path).copied().unwrap_or(0)
    }

    pub fn get_fan_out(&self, path: &Path) -> usize {
        self.fan_out.get(path).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::FileStructure;

    fn make_s(path: &str, pkg: &str, module: &str, use_paths: Vec<&str>) -> FileStructure {
        FileStructure {
            path: PathBuf::from(path),
            relative_path: PathBuf::from(path),
            package_name: pkg.to_string(),
            module_path: format!("{}::{}", pkg, module),
            use_paths: use_paths.into_iter().map(String::from).collect(),
            pub_use_paths: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn crate_prefix_resolves_nested_module() {
        let ss = vec![
            make_s("src/main.rs", "my_crate", "main", vec!["crate::discovery::files::RustFile"]),
            make_s("src/discovery/files.rs", "my_crate", "discovery::files", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_in(&PathBuf::from("src/discovery/files.rs")), 1);
        assert_eq!(g.get_fan_out(&PathBuf::from("src/main.rs")), 1);
    }

    #[test]
    fn super_prefix_resolves_relative_to_parent() {
        // src/api/child.rs (module my_crate::api::child) uses super::helpers::Bar
        // super:: should walk up to my_crate::api, then look for my_crate::api::helpers
        let ss = vec![
            make_s("src/api/child.rs", "my_crate", "api::child", vec!["super::helpers::Bar"]),
            make_s("src/api/helpers.rs", "my_crate", "api::helpers", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_in(&PathBuf::from("src/api/helpers.rs")), 1);
    }

    #[test]
    fn super_prefix_does_not_resolve_to_root_helpers() {
        // Regression: super:: should not resolve to top-level "helpers" module
        // when the file is nested inside a package
        let ss = vec![
            make_s("src/api/child.rs", "my_crate", "api::child", vec!["super::helpers::Bar"]),
            make_s("src/helpers.rs", "my_crate", "helpers", vec![]),
            make_s("src/api/helpers.rs", "my_crate", "api::helpers", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        // Should resolve to src/api/helpers.rs, NOT src/helpers.rs
        assert_eq!(g.get_fan_in(&PathBuf::from("src/api/helpers.rs")), 1);
        assert_eq!(g.get_fan_in(&PathBuf::from("src/helpers.rs")), 0);
    }

    #[test]
    fn self_prefix_resolves_from_current_module() {
        let ss = vec![
            make_s("src/utils.rs", "my_crate", "utils", vec!["self::internal::Foo"]),
            make_s("src/utils/internal.rs", "my_crate", "utils::internal", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_in(&PathBuf::from("src/utils/internal.rs")), 1);
    }

    #[test]
    fn mod_rs_resolves_correctly() {
        let ss = vec![
            make_s("src/main.rs", "my_crate", "main", vec!["crate::api::Handler"]),
            make_s("src/api/mod.rs", "my_crate", "api", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_in(&PathBuf::from("src/api/mod.rs")), 1);
    }

    #[test]
    fn self_import_not_counted() {
        let ss = vec![make_s("src/utils.rs", "my_crate", "utils", vec!["crate::utils::internal"])];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_in(&PathBuf::from("src/utils.rs")), 0);
    }

    #[test]
    fn external_crate_ignored() {
        let ss = vec![
            make_s("src/lib.rs", "my_crate", "lib", vec!["std::collections::HashMap", "serde::Serialize"]),
            make_s("src/utils.rs", "my_crate", "utils", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("src"));
        assert_eq!(g.get_fan_out(&PathBuf::from("src/lib.rs")), 0);
    }

    #[test]
    fn no_collision_across_workspace_packages() {
        // Two packages both have src/utils.rs → utils, but qualified as pkg_a::utils and pkg_b::utils
        let ss = vec![
            make_s("pkg_a/src/main.rs", "pkg_a", "main", vec!["crate::utils::Foo"]),
            make_s("pkg_a/src/utils.rs", "pkg_a", "utils", vec![]),
            make_s("pkg_b/src/main.rs", "pkg_b", "main", vec!["crate::utils::Bar"]),
            make_s("pkg_b/src/utils.rs", "pkg_b", "utils", vec![]),
        ];
        let g = ImportGraph::build(&ss, Path::new("."));
        // Each package's main only increments its own utils
        assert_eq!(g.get_fan_in(&PathBuf::from("pkg_a/src/utils.rs")), 1);
        assert_eq!(g.get_fan_in(&PathBuf::from("pkg_b/src/utils.rs")), 1);
    }
}
