use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::discovery::workspace::PackageInfo;

#[derive(Debug, Clone)]
pub struct RustFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub package_name: String,
    pub size_bytes: u64,
    pub line_count: usize,
}

/// Walk each package's declared source roots (from cargo metadata targets),
/// rather than the entire workspace tree. This avoids pulling in `.rs` files
/// outside real compilation units (e.g., docs, examples not listed as targets).
pub fn enumerate_rust_files(
    packages: &[PackageInfo],
    workspace_root: &Path,
    include_tests: bool,
) -> Vec<RustFile> {
    let mut files = Vec::new();

    for pkg in packages {
        for root in &pkg.source_roots {
            if !root.exists() {
                continue;
            }
            let pkg_files = walk_source_root(root, workspace_root, &pkg.name, include_tests);
            files.extend(pkg_files);
        }
    }

    // Deduplicate by canonical path in case source roots overlap
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    files
}

fn walk_source_root(
    root: &Path,
    workspace_root: &Path,
    package_name: &str,
    include_tests: bool,
) -> Vec<RustFile> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            if p.extension().is_none_or(|ext| ext != "rs") {
                return false;
            }
            if p.components().any(|c| c.as_os_str() == "target") {
                return false;
            }
            if !include_tests {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                if name.ends_with("_test.rs") || name == "tests.rs" {
                    return false;
                }
                if p.components().any(|c| c.as_os_str() == "tests") {
                    return false;
                }
            }
            true
        })
        .filter_map(|e| {
            let path = e.path().to_path_buf();
            let meta = std::fs::metadata(&path).ok()?;
            let content = std::fs::read_to_string(&path).ok()?;
            let line_count = content.lines().count();
            // Relative to workspace root for display; fall back to file name
            let relative_path = path
                .strip_prefix(workspace_root)
                .unwrap_or(&path)
                .to_path_buf();
            Some(RustFile {
                path,
                relative_path,
                package_name: package_name.to_string(),
                size_bytes: meta.len(),
                line_count,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::workspace::PackageInfo;
    use std::fs;
    use tempfile::TempDir;

    fn mkpkg(dir: &Path, name: &str) -> PackageInfo {
        PackageInfo {
            name: name.to_string(),
            manifest_path: dir.join("Cargo.toml"),
            source_roots: vec![dir.to_path_buf()],
        }
    }

    fn mkrs(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn enumerates_rs_files() {
        let tmp = TempDir::new().unwrap();
        mkrs(tmp.path(), "lib.rs", "fn foo() {}");
        mkrs(tmp.path(), "main.rs", "fn main() {}");
        let pkgs = vec![mkpkg(tmp.path(), "test-crate")];
        assert_eq!(enumerate_rust_files(&pkgs, tmp.path(), false).len(), 2);
    }

    #[test]
    fn counts_lines() {
        let tmp = TempDir::new().unwrap();
        mkrs(tmp.path(), "lib.rs", "a\nb\nc\n");
        let pkgs = vec![mkpkg(tmp.path(), "test-crate")];
        assert_eq!(enumerate_rust_files(&pkgs, tmp.path(), false)[0].line_count, 3);
    }

    #[test]
    fn skips_target() {
        let tmp = TempDir::new().unwrap();
        let tgt = tmp.path().join("target/debug");
        fs::create_dir_all(&tgt).unwrap();
        mkrs(&tgt, "gen.rs", "fn x() {}");
        mkrs(tmp.path(), "lib.rs", "fn y() {}");
        let pkgs = vec![mkpkg(tmp.path(), "test-crate")];
        let files = enumerate_rust_files(&pkgs, tmp.path(), false);
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn excludes_test_files_by_default() {
        let tmp = TempDir::new().unwrap();
        mkrs(tmp.path(), "lib.rs", "fn r() {}");
        mkrs(tmp.path(), "lib_test.rs", "fn t() {}");
        let pkgs = vec![mkpkg(tmp.path(), "test-crate")];
        assert_eq!(enumerate_rust_files(&pkgs, tmp.path(), false).len(), 1);
    }

    #[test]
    fn includes_test_files_when_flag_set() {
        let tmp = TempDir::new().unwrap();
        mkrs(tmp.path(), "lib.rs", "fn r() {}");
        mkrs(tmp.path(), "lib_test.rs", "fn t() {}");
        let pkgs = vec![mkpkg(tmp.path(), "test-crate")];
        assert_eq!(enumerate_rust_files(&pkgs, tmp.path(), true).len(), 2);
    }

    #[test]
    fn deduplicates_overlapping_roots() {
        let tmp = TempDir::new().unwrap();
        mkrs(tmp.path(), "lib.rs", "fn r() {}");
        // Same root listed twice in different packages should not duplicate
        let pkgs = vec![
            PackageInfo {
                name: "a".to_string(),
                manifest_path: tmp.path().join("Cargo.toml"),
                source_roots: vec![tmp.path().to_path_buf()],
            },
            PackageInfo {
                name: "b".to_string(),
                manifest_path: tmp.path().join("Cargo.toml"),
                source_roots: vec![tmp.path().to_path_buf()],
            },
        ];
        assert_eq!(enumerate_rust_files(&pkgs, tmp.path(), false).len(), 1);
    }
}
