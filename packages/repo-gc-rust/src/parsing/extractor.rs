use std::collections::HashSet;
use std::path::{Path, PathBuf};
use anyhow::Result;
use quote::quote;
use syn::visit::Visit;

#[derive(Debug, Clone, Default)]
pub struct FileStructure {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub relative_path: PathBuf,
    #[allow(dead_code)]
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
    /// Leaf names imported by private `use` statements (for unused-import detection)
    pub use_leaf_names: Vec<String>,
    /// All (name, is_public) for fns and methods (for duplication + dead-private detection)
    pub function_names: Vec<(String, bool)>,
    /// (fn_name, normalized_body_tokens) for duplication detection
    pub function_bodies: Vec<(String, String)>,
    /// All identifiers used in the file body, excluding `use` declarations
    pub all_identifiers: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct PubUseEntry {
    pub path: String,
    pub item_count: usize,
    pub is_wildcard: bool,
}

/// Compute the unqualified Rust module path from a path relative to the workspace root.
///
/// Works for any layout — finds the "src" component positionally, then uses
/// everything after it. Falls back to the whole path if no "src" is present.
///
/// src/foo/bar.rs                    → "foo::bar"
/// src/foo/mod.rs                    → "foo"
/// src/lib.rs                        → "lib"
/// packages/repo-gc-rust/src/foo.rs  → "foo"
pub fn module_path_from_relative(rel: &Path) -> String {
    let components: Vec<_> = rel.components().collect();

    // Find the "src" component; use everything after it (or the whole path as fallback)
    let src_pos = components.iter().position(|c| {
        matches!(c, std::path::Component::Normal(s) if *s == "src")
    });
    let after_src = if let Some(pos) = src_pos {
        &components[pos + 1..]
    } else {
        &components[..]
    };

    let mut parts: Vec<&str> = after_src
        .iter()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();

    // Drop file extension from the last segment
    if let Some(last) = parts.last_mut() {
        if let Some(dot) = last.rfind('.') {
            *last = &last[..dot];
        }
    }

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
        let is_pub = matches!(i.vis, syn::Visibility::Public(_));
        if is_pub {
            self.s.public_function_count += 1;
        }
        let name = i.sig.ident.to_string();
        self.s.function_names.push((name.clone(), is_pub));
        let body = i.block.as_ref();
        self.s.function_bodies.push((name, quote!(#body).to_string()));
        syn::visit::visit_item_fn(self, i);
    }
    fn visit_impl_item_fn(&mut self, i: &'ast syn::ImplItemFn) {
        self.s.function_count += 1;
        let is_pub = matches!(i.vis, syn::Visibility::Public(_));
        if is_pub {
            self.s.public_function_count += 1;
        }
        let name = i.sig.ident.to_string();
        self.s.function_names.push((name.clone(), is_pub));
        let body = &i.block;
        self.s.function_bodies.push((name, quote!(#body).to_string()));
        syn::visit::visit_impl_item_fn(self, i);
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
            self.s.use_leaf_names.extend(collect_leaf_names(&i.tree));
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

/// Collect the imported leaf names from a use tree.
/// `use a::b::{C, D as E}` → ["C", "E"]
/// Wildcards produce no names (can't know what they import).
fn collect_leaf_names(t: &syn::UseTree) -> Vec<String> {
    match t {
        syn::UseTree::Path(p) => collect_leaf_names(&p.tree),
        syn::UseTree::Name(n) => vec![n.ident.to_string()],
        syn::UseTree::Rename(r) => vec![r.rename.to_string()],
        syn::UseTree::Glob(_) => vec![],
        syn::UseTree::Group(g) => g.items.iter().flat_map(collect_leaf_names).collect(),
    }
}

/// Collects all identifiers used in the file, skipping `use` declarations.
/// Used to detect which imported names are actually referenced.
#[derive(Default)]
struct IdentCollector {
    idents: HashSet<String>,
}

impl<'ast> Visit<'ast> for IdentCollector {
    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        self.idents.insert(i.to_string());
    }
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}
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
    let mut s = visitor.s;
    let mut ic = IdentCollector::default();
    ic.visit_file(&syntax);
    s.all_identifiers = ic.idents;
    Ok(s)
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
