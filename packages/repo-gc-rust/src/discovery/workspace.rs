use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub manifest_path: PathBuf,
    /// Source roots for this package, derived from Cargo target src paths.
    pub source_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub packages: Vec<PackageInfo>,
    pub is_workspace: bool,
}

pub fn discover_workspace(path: &Path) -> Result<WorkspaceInfo> {
    let canonical = path.canonicalize()?;
    let meta_result = cargo_metadata::MetadataCommand::new()
        .current_dir(&canonical)
        .exec();

    match meta_result {
        Ok(meta) => {
            let packages: Vec<PackageInfo> = meta
                .workspace_packages()
                .into_iter()
                .map(|pkg| {
                    let manifest_dir = pkg
                        .manifest_path
                        .parent()
                        .map(|p| p.to_path_buf().into_std_path_buf())
                        .unwrap_or_else(|| {
                            meta.workspace_root.clone().into_std_path_buf()
                        });

                    // Collect unique source directories from Cargo targets
                    let mut source_roots: Vec<PathBuf> = pkg
                        .targets
                        .iter()
                        .filter_map(|t| {
                            t.src_path
                                .parent()
                                .map(|p| p.to_path_buf().into_std_path_buf())
                        })
                        .collect();
                    source_roots.sort();
                    source_roots.dedup();

                    if source_roots.is_empty() {
                        source_roots.push(manifest_dir.join("src"));
                    }

                    PackageInfo {
                        name: pkg.name.clone(),
                        manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                        source_roots,
                    }
                })
                .collect();

            let is_workspace = packages.len() > 1;
            Ok(WorkspaceInfo {
                root: meta.workspace_root.into_std_path_buf(),
                packages,
                is_workspace,
            })
        }
        Err(_) => {
            // Not a Cargo workspace — treat the directory itself as a single package
            let src = canonical.join("src");
            let source_roots = if src.exists() {
                vec![src]
            } else {
                vec![canonical.clone()]
            };
            Ok(WorkspaceInfo {
                root: canonical.clone(),
                packages: vec![PackageInfo {
                    name: "unknown".to_string(),
                    manifest_path: canonical.join("Cargo.toml"),
                    source_roots,
                }],
                is_workspace: false,
            })
        }
    }
}
