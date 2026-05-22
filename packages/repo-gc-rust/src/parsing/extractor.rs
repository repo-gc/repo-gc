use std::collections::HashSet;
use std::path::{Path, PathBuf};
use anyhow::Result;
use quote::quote;
use syn::visit::Visit;
use syn::{BinOp, Expr, Pat};

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
    pub branch_count: usize,
    pub max_nesting_depth: usize,
    pub max_type_depth: usize,
    pub comment_line_count: usize,
    pub decorator_count: usize,
    pub empty_catch_count: usize,
    pub dangerous_pattern_count: usize,
    pub string_comparison_count: usize,
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

/// Check if a file path indicates a test file.
fn is_test_file(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();
    if s.contains("/tests/") || s.contains("\\tests\\") {
        return true;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let name_lower = name.to_lowercase();
        if name_lower.contains("test") || name_lower.contains("spec") {
            return true;
        }
    }
    false
}

struct Visitor {
    s: FileStructure,
    type_depth: usize,
    depth: usize,
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
            depth: 0,
            type_depth: 0,
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // Count dangerous patterns and string comparisons
        match node {
            Expr::Unsafe(_) => {
                self.s.dangerous_pattern_count += 1;
            }
            Expr::MethodCall(mc) => {
                // Skip unwrap/expect in test files (idiomatic in tests)
                if !is_test_file(&self.s.relative_path) {
                    let method_name = mc.method.to_string();
                    if method_name == "unwrap" || method_name == "expect" {
                        self.s.dangerous_pattern_count += 1;
                    }
                }
            }
            Expr::Call(call) => {
                // Detect transmute calls (e.g. transmute(x) or std::mem::transmute(x))
                if let Expr::Path(ref p) = *call.func {
                    if let Some(seg) = p.path.segments.last() {
                        if seg.ident == "transmute" {
                            self.s.dangerous_pattern_count += 1;
                        }
                    }
                }
            }
            Expr::Binary(bin) => {
                // Count string literal comparisons: `x == "foo"` or `"bar" != y`
                if matches!(bin.op, BinOp::Eq(_) | BinOp::Ne(_)) {
                    if is_string_literal(&bin.left) || is_string_literal(&bin.right) {
                        self.s.string_comparison_count += 1;
                    }
                }
            }
            _ => {}
        }

        let is_block_like = matches!(node,
            syn::Expr::If(_) | syn::Expr::Match(_) | syn::Expr::ForLoop(_) |
            syn::Expr::While(_) | syn::Expr::Loop(_) |
            syn::Expr::Block(_) | syn::Expr::Closure(_) |
            syn::Expr::Unsafe(_)
        );
        if is_block_like {
            self.depth += 1;
            self.s.max_nesting_depth = self.s.max_nesting_depth.max(self.depth);
        }
        syn::visit::visit_expr(self, node);
        if is_block_like {
            self.depth -= 1;
        }
    }
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        self.s.function_count += 1;
        self.s.decorator_count += i.attrs.len();
        let is_pub = matches!(i.vis, syn::Visibility::Public(_));
        if is_pub {
            self.s.public_function_count += 1;
        }
        let name = i.sig.ident.to_string();
        self.s.function_names.push((name.clone(), is_pub));
        let body = i.block.as_ref();
        self.s.function_bodies.push((name, quote!(#body).to_string()));
        self.depth += 1;
        self.s.max_nesting_depth = self.s.max_nesting_depth.max(self.depth);
        syn::visit::visit_item_fn(self, i);
        self.depth -= 1;
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
        self.depth += 1;
        self.s.max_nesting_depth = self.s.max_nesting_depth.max(self.depth);
        syn::visit::visit_impl_item_fn(self, i);
        self.depth -= 1;
    }
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.s.impl_block_count += 1;
        self.s.decorator_count += i.attrs.len();
        self.depth += 1;
        self.s.max_nesting_depth = self.s.max_nesting_depth.max(self.depth);
        syn::visit::visit_item_impl(self, i);
        self.depth -= 1;
    }
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        self.s.struct_count += 1;
        self.s.decorator_count += i.attrs.len();
    }
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        self.s.enum_count += 1;
        self.s.decorator_count += i.attrs.len();
    }
    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        self.s.trait_count += 1;
        self.s.decorator_count += i.attrs.len();
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
    fn visit_expr_match(&mut self, expr: &'ast syn::ExprMatch) {
        self.s.branch_count += expr.arms.len();
        for arm in &expr.arms {
            if is_err_pat(&arm.pat) && is_empty_block_expr(arm.body.as_ref()) {
                self.s.empty_catch_count += 1;
            }
        }
        syn::visit::visit_expr_match(self, expr);
    }
    fn visit_expr_if(&mut self, expr: &'ast syn::ExprIf) {
        self.s.branch_count += 1;
        // Detect `if let Err(_) = expr {}` — an if-let with empty then-branch
        if expr.then_branch.stmts.is_empty() && expr.else_branch.is_none() {
            if let Expr::Let(let_expr) = expr.cond.as_ref() {
                if is_err_pat(let_expr.pat.as_ref()) {
                    self.s.empty_catch_count += 1;
                }
            }
        }
        syn::visit::visit_expr_if(self, expr);
    }
    fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
        self.s.branch_count += 1;
    }
    fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
        self.s.branch_count += 1;
    }
    fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {
        self.s.branch_count += 1;
    }
    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        // Count string literals in match arm patterns: `match x { "foo" => ... }`
        if let Pat::Lit(lit) = node {
            if matches!(&lit.lit, syn::Lit::Str(_)) {
                self.s.string_comparison_count += 1;
            }
        }
        syn::visit::visit_pat(self, node);
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        let inc = match ty {
            syn::Type::Path(tp) => tp.path.segments.iter().any(|seg| {
                matches!(&seg.arguments, syn::PathArguments::AngleBracketed(args) if !args.args.is_empty())
            }),
            syn::Type::Tuple(_) | syn::Type::Array(_)
            | syn::Type::Reference(_) | syn::Type::Slice(_) | syn::Type::Ptr(_) => true,
            _ => false,
        };
        if inc {
            self.type_depth += 1;
            self.s.max_type_depth = self.s.max_type_depth.max(self.type_depth);
        }
        syn::visit::visit_type(self, ty);
        if inc {
            self.type_depth -= 1;
        }
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

/// Check if a pattern is `Err(_)` — an error binding that discards the payload.
fn is_err_pat(pat: &Pat) -> bool {
    match pat {
        Pat::TupleStruct(pts) => {
            pts.path.segments.last().map_or(false, |s| s.ident == "Err")
                && pts.elems.iter().any(|p| matches!(p, Pat::Wild(_)))
        }
        _ => false,
    }
}

/// Check if an expression is an empty block `{}`.
fn is_empty_block_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Block(block) if block.block.stmts.is_empty())
}

/// Check if an expression is a string literal.
fn is_string_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(_)))
}

/// Count comment lines from raw source text before syn strips them.
///
/// Counts:
/// - Lines where the first non-whitespace content is `//`
/// - Lines inside `/* ... */` block comments
fn count_comment_lines(source: &str) -> usize {
    let mut count = 0;
    let mut in_block = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if in_block {
            count += 1;
            if trimmed.contains("*/") {
                in_block = false;
            }
        } else if trimmed.starts_with("//") {
            count += 1;
        } else if let Some(pos) = trimmed.find("/*") {
            count += 1;
            // Check if the block comment ends on the same line
            let after = &trimmed[pos + 2..];
            if !after.contains("*/") {
                in_block = true;
            }
        }
    }
    count
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
    // Count comment lines from raw source BEFORE syn strips them
    let comment_line_count = count_comment_lines(&content);
    let syntax = syn::parse_file(&content)?;
    let mut visitor = Visitor::new(path.to_path_buf(), relative_path, package_name);
    visitor.visit_file(&syntax);
    let mut s = visitor.s;
    s.comment_line_count = comment_line_count;
    // Count .ok() calls in raw source (best-effort, no type resolution)
    s.empty_catch_count += content.matches(".ok(").count();
    // Apply 0.5x multiplier for test files (dangerous patterns are more
    // acceptable in test code, except unwrap/expect which are skipped above)
    if is_test_file(&s.relative_path) {
        s.dangerous_pattern_count = (s.dangerous_pattern_count as f64 * 0.5).round() as usize;
    }
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

    // -- dangerous pattern counting tests -------------------------------------

    #[test]
    fn counts_unsafe_blocks() {
        let s = parse("fn f() { unsafe { let x = 1; } }", "src/lib.rs", "krate");
        assert_eq!(s.dangerous_pattern_count, 1);
    }

    #[test]
    fn counts_unwrap_calls() {
        let s = parse("fn f() { let x = Some(1).unwrap(); }", "src/lib.rs", "krate");
        assert_eq!(s.dangerous_pattern_count, 1);
    }

    #[test]
    fn counts_expect_calls() {
        let s = parse("fn f() { let x = Some(1).expect(\"msg\"); }", "src/lib.rs", "krate");
        assert_eq!(s.dangerous_pattern_count, 1);
    }

    #[test]
    fn counts_transmute_calls() {
        let s = parse(
            "fn f() { let x: u32 = unsafe { std::mem::transmute(1.0f32) }; }",
            "src/lib.rs",
            "krate",
        );
        assert_eq!(s.dangerous_pattern_count, 2); // unsafe + transmute
    }

    #[test]
    fn no_unwrap_in_test_files() {
        // test file path — unwrap should not be counted
        let s = parse("fn f() { let x = Some(1).unwrap(); }", "src/test_foo.rs", "krate");
        assert_eq!(s.dangerous_pattern_count, 0);
    }

    #[test]
    fn test_file_multiplier_applied() {
        // test file with 2 unsafe blocks — 2 * 0.5 = 1
        let s = parse(
            "fn f() { unsafe { let x = 1; } fn g() { unsafe { let y = 2; } } }",
            "src/tests/foo.rs",
            "krate",
        );
        assert_eq!(s.dangerous_pattern_count, 1); // 2 * 0.5 = 1
    }

    #[test]
    fn no_dangerous_patterns_in_clean_code() {
        let s = parse("fn add(a: i32, b: i32) -> i32 { a + b }", "src/lib.rs", "krate");
        assert_eq!(s.dangerous_pattern_count, 0);
    }

    #[test]
    fn is_test_file_detects_test_in_name() {
        assert!(is_test_file(Path::new("src/test_foo.rs")));
        assert!(is_test_file(Path::new("src/foo_test.rs")));
        assert!(is_test_file(Path::new("src/tests/foo.rs")));
        assert!(!is_test_file(Path::new("src/lib.rs")));
        assert!(!is_test_file(Path::new("src/main.rs")));
        assert!(!is_test_file(Path::new("src/foo.rs")));
    }
}
