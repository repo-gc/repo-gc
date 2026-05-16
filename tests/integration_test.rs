use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn repo_gc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_repo-gc"))
}

fn make_test_repo(tmp: &TempDir) {
    fs::write(
        tmp.path().join("Cargo.toml"),
        r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::create_dir(tmp.path().join("src")).unwrap();

    // Context bomb: 600 pub functions
    let big: String = (0..600)
        .map(|i| format!("pub fn f{}() -> u32 {{ {} }}\n", i, i))
        .collect();
    fs::write(tmp.path().join("src/big.rs"), big).unwrap();

    // Barrel file with many pub use + mod big
    fs::write(
        tmp.path().join("src/lib.rs"),
        r#"
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::collections::BTreeMap;
pub use std::sync::Arc;
pub use std::sync::Mutex;
pub use std::sync::RwLock;
pub use std::io::Read;
pub use std::io::Write;
pub use std::io::BufReader;
pub use std::io::BufWriter;
mod big;
"#,
    )
    .unwrap();
}

#[test]
fn scan_produces_output_on_test_repo() {
    let tmp = TempDir::new().unwrap();
    make_test_repo(&tmp);

    let out = repo_gc()
        .args(["scan", "--path", tmp.path().to_str().unwrap(), "--no-color"])
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Findings") || stdout.contains("No issues"),
        "output: {}",
        stdout
    );
}

#[test]
fn json_output_has_expected_fields() {
    let tmp = TempDir::new().unwrap();
    make_test_repo(&tmp);

    let out = repo_gc()
        .args(["scan", "--format", "json", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON: {} — output was: {}", e, stdout));
    assert!(v["findings"].is_array(), "missing findings");
    assert!(
        v["global_score"]["ai_hostility_score"].is_number(),
        "missing ai_hostility_score"
    );
    assert!(v["files_analyzed"].is_number(), "missing files_analyzed");
    assert!(v["files_skipped"].is_number(), "missing files_skipped");
}

#[test]
fn strict_threshold_finds_at_least_as_many_as_relaxed() {
    let tmp = TempDir::new().unwrap();
    make_test_repo(&tmp);

    let count = |t: &str| -> usize {
        let out = repo_gc()
            .args([
                "scan",
                "--format",
                "json",
                "--threshold",
                t,
                "--path",
                tmp.path().to_str().unwrap(),
            ])
            .output()
            .unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
                .unwrap_or(serde_json::json!({"findings": []}));
        v["findings"].as_array().map(|a| a.len()).unwrap_or(0)
    };

    let strict = count("strict");
    let relaxed = count("relaxed");
    assert!(
        strict >= relaxed,
        "strict={} should be >= relaxed={}",
        strict,
        relaxed
    );
}

#[test]
fn empty_dir_returns_zero_findings() {
    let tmp = TempDir::new().unwrap();
    let out = repo_gc()
        .args(["scan", "--format", "json", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["findings"].as_array().unwrap().len(), 0);
    assert_eq!(v["files_analyzed"], 0);
}
