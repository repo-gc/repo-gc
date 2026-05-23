//! Shared test helpers for heuristic unit tests.
//!
//! Import this instead of defining local `fn f()` and `fn s()` in every test module.

#![allow(dead_code)]

use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use std::path::PathBuf;

pub fn test_file(name: &str) -> RustFile {
    RustFile {
        path: PathBuf::from(name),
        relative_path: PathBuf::from(name),
        package_name: "krate".to_string(),
        size_bytes: 100,
        line_count: 10,
    }
}

pub fn test_file_sized(name: &str, line_count: usize, size_bytes: u64) -> RustFile {
    RustFile {
        path: PathBuf::from(name),
        relative_path: PathBuf::from(name),
        package_name: "krate".to_string(),
        size_bytes,
        line_count,
    }
}

pub fn test_structure(path: &str) -> FileStructure {
    FileStructure {
        path: PathBuf::from(path),
        ..Default::default()
    }
}
