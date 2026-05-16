use std::path::{Path, PathBuf};
use anyhow::Result;
use syn::visit::Visit;

#[derive(Debug, Clone, Default)]
pub struct FileStructure {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub package_name: String,
    /// Package-qualified Rust module path, e.g. "my_crate::foo::bar" for src/foo/bar.rs
    pub module_path: String,
    pub function_count: usize,
    pub public_function_count: usize,
    pub impl_block_count: usize,
    pub struct_count: usize,
    pub enum_count: usize,
    pub trait_count: usize,
    pub use_paths: Vec<String>,
    pub pub_use_paths: Vec<PubUseEntry>,
    pub mod_declarations: Vec<String>,
    pub pub_mod_count: usize,
}

#[derive(Debug, Clone)]
pub struct PubUseEntry {
    pub path: String,
    pub item_count: usize,
    pub is_wildcard: bool,
}

/// Compute the unqualified Rust module path from a path relative to the workspace root.
///
/// src/foo/bar.rs  → "foo::bar"
/// src/foo/mod.rs  → "foo"
/// src/lib.rs      → "lib"
/// foo/bar.rs      → "foo::bar"  (no src/ prefix)
pub fn module_path_from_relative(rel: &Path) -> String {
    // Strip leading "src" component if present
    let stripped = if rel.starts_with("src") {
        rel.strip_prefix("src").unwrap_or(rel)
    } else {
        rel
    };

    let without_ext = stripped.with_extension("");
    let parts: Vec<&str> = without_ext
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();

    if parts.is_empty() {
        return String::new();
    }

    // Drop trailing "mod" — src/foo/mod.rs becomes module "foo"
    let effective = if parts.last() == Some(&"mod") {
        &parts[..parts.len() - 1]
    } else {
        &parts[..]
    };

    effective.join("::")
}

/// Build a package-qualified module path to avoid collisions in multi-crate workspaces.
/// "my_crate" + "foo::bar" → "my_crate::foo::bar"
pub fn qualified_module_path(package_name: &str, unqualified: &str) -> String {
    if unqualified.is_empty() {
        package_name.to_string()
    } else {
        format!("{}::{}", package_name, unqualified)
    }
}

struct Visitor {
    s: FileStructure,
}

impl Visitor {
    fn new(path: PathBuf, relative_path: PathBuf, package_name: String) -> Self {
        let unqualified = module_path_from_relative(&relative_path);
        let module_path = qualified_module_path(&package_name, &unqualified);
        Self {
            s: FileStructure {
                path,
                relative_path,
                package_name,
                module_path,
                ..Default::default()
            },
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        self.s.function_count += 1;
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.s.public_function_count += 1;
        }
        syn::visit::visit_item_fn(self, i);
    }
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.s.impl_block_count += 1;
        syn::visit::visit_item_impl(self, i);
    }
    fn visit_item_struct(&mut self, _: &'ast syn::ItemStruct) {
        self.s.struct_count += 1;
    }
    fn visit_item_enum(&mut self, _: &'ast syn::ItemEnum) {
        self.s.enum_count += 1;
    }
    fn visit_item_trait(&mut self, _: &'ast syn::ItemTrait) {
        self.s.trait_count += 1;
    }
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        let path_str = use_tree_to_string(&i.tree);
        let is_pub = matches!(i.vis, syn::Visibility::Public(_));
        let item_count = count_use_items(&i.tree);
        let is_wildcard = is_wildcard_use(&i.tree);
        if is_pub {
            self.s.pub_use_paths.push(PubUseEntry {
                path: path_str,
                item_count,
                is_wildcard,
            });
        } else {
            self.s.use_paths.push(path_str);
        }
    }
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        // Record fully qualified mod declaration: current module path + child name
        // e.g. file at "my_crate::api" declaring `mod handlers` → "my_crate::api::handlers"
        let child_mod = if self.s.module_path.is_empty() {
            i.ident.to_string()
        } else {
            format!("{}::{}", self.s.module_path, i.ident)
        };
        self.s.mod_declarations.push(child_mod);
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.s.pub_mod_count += 1;
        }
        syn::visit::visit_item_mod(self, i);
    }
}

fn use_tree_to_string(t: &syn::UseTree) -> String {
    match t {
        syn::UseTree::Path(p) => format!("{}::{}", p.ident, use_tree_to_string(&p.tree)),
        syn::UseTree::Name(n) => n.ident.to_string(),
        syn::UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        syn::UseTree::Glob(_) => "*".to_string(),
        syn::UseTree::Group(g) => {
            let items: Vec<_> = g.items.iter().map(use_tree_to_string).collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn count_use_items(t: &syn::UseTree) -> usize {
    match t {
        syn::UseTree::Path(p) => count_use_items(&p.tree),
        syn::UseTree::Group(g) => g.items.iter().map(count_use_items).sum(),
        _ => 1,
    }
}

fn is_wildcard_use(t: &syn::UseTree) -> bool {
    match t {
        syn::UseTree::Path(p) => is_wildcard_use(&p.tree),
        syn::UseTree::Glob(_) => true,
        _ => false,
    }
}

pub fn extract_structure(
    path: &Path,
    relative_path: PathBuf,
    package_name: String,
) -> Result<FileStructure> {
    let content = std::fs::read_to_string(path)?;
    let syntax = syn::parse_file(&content)?;
    let mut visitor = Visitor::new(path.to_path_buf(), relative_path, package_name);
    visitor.visit_file(&syntax);
    Ok(visitor.s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn parse(code: &str, rel: &str, pkg: &str) -> FileStructure {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("test.rs");
        fs::write(&p, code).unwrap();
        extract_structure(&p, PathBuf::from(rel), pkg.to_string()).unwrap()
    }

    #[test]
    fn module_path_flat_file() {
        assert_eq!(module_path_from_relative(Path::new("src/foo.rs")), "foo");
    }

    #[test]
    fn module_path_nested() {
        assert_eq!(module_path_from_relative(Path::new("src/foo/bar.rs")), "foo::bar");
    }

    #[test]
    fn module_path_mod_rs_drops_mod() {
        assert_eq!(module_path_from_relative(Path::new("src/foo/mod.rs")), "foo");
    }

    #[test]
    fn module_path_without_src_prefix() {
        assert_eq!(module_path_from_relative(Path::new("foo/bar.rs")), "foo::bar");
    }

    #[test]
    fn qualified_path_prepends_package() {
        assert_eq!(qualified_module_path("my_crate", "foo::bar"), "my_crate::foo::bar");
    }

    #[test]
    fn qualified_path_empty_unqualified() {
        assert_eq!(qualified_module_path("my_crate", ""), "my_crate");
    }

    #[test]
    fn counts_functions_and_visibility() {
        let s = parse("pub fn foo() {} fn bar() {} fn baz() {}", "src/lib.rs", "krate");
        assert_eq!(s.function_count, 3);
        assert_eq!(s.public_function_count, 1);
    }

    #[test]
    fn counts_impl_blocks() {
        let s = parse(
            "struct F; impl F {} impl Drop for F { fn drop(&mut self) {} }",
            "src/lib.rs",
            "krate",
        );
        assert_eq!(s.impl_block_count, 2);
    }

    #[test]
    fn detects_pub_use_and_wildcard() {
        let s = parse(
            "pub use std::collections::HashMap; pub use crate::foo::*;",
            "src/lib.rs",
            "krate",
        );
        assert_eq!(s.pub_use_paths.len(), 2);
        assert!(s.pub_use_paths.iter().any(|u| u.is_wildcard));
    }

    #[test]
    fn counts_grouped_pub_use_items() {
        let s = parse(
            "pub use std::{collections::HashMap, sync::Arc, io::Write};",
            "src/lib.rs",
            "krate",
        );
        assert_eq!(s.pub_use_paths[0].item_count, 3);
    }

    #[test]
    fn mod_declarations_are_fully_qualified() {
        let s = parse("pub mod alpha; mod beta;", "src/lib.rs", "my_crate");
        // lib.rs has module_path "my_crate::lib", so children should be "my_crate::lib::alpha"
        assert!(s.mod_declarations.iter().any(|m| m.ends_with("::alpha")));
        assert!(s.mod_declarations.iter().any(|m| m.ends_with("::beta")));
        assert_eq!(s.pub_mod_count, 1);
    }

    #[test]
    fn structure_carries_package_qualified_module_path() {
        let s = parse("fn x() {}", "src/foo/bar.rs", "my_crate");
        assert_eq!(s.module_path, "my_crate::foo::bar");
    }
}
