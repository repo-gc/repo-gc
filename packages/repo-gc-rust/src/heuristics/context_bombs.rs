use crate::cli::Threshold;
use crate::discovery::RustFile;
use crate::parsing::FileStructure;
use crate::types::{next_finding_id, Finding, FindingKind, Severity};

/// 4 bytes ≈ 1 token (LLM tokenizer heuristic).
pub fn estimate_tokens(size_bytes: u64) -> usize {
    (size_bytes as usize).saturating_div(4)
}

pub fn analyze(
    file: &RustFile,
    structure: &FileStructure,
    threshold: &Threshold,
    counter: &mut usize,
) -> Option<Finding> {
    let limit = threshold.line_count_limit();
    if file.line_count < limit {
        return None;
    }

    let severity = if file.line_count >= limit * 4 {
        Severity::Critical
    } else if file.line_count >= limit * 2 {
        Severity::High
    } else {
        Severity::Medium
    };

    let estimated_tokens = estimate_tokens(file.size_bytes);

    let mut evidence = vec![
        format!("line_count: {}", file.line_count),
        format!("estimated_tokens: {}", estimated_tokens),
        format!("limit: {}", limit),
    ];
    if structure.function_count > 10 {
        evidence.push(format!("function_count: {}", structure.function_count));
    }
    if structure.impl_block_count > 3 {
        evidence.push(format!("impl_block_count: {}", structure.impl_block_count));
    }

    let mut finding = Finding::new(
        next_finding_id("cb", counter),
        FindingKind::ContextBomb,
        severity,
        if file.line_count >= limit * 4 {
            0.95
        } else if file.line_count >= limit * 2 {
            0.85
        } else {
            0.75
        },
        file.relative_path.clone(),
        evidence,
    );
    finding.estimated_tokens = Some(estimated_tokens);
    Some(finding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn f(lines: usize, bytes: u64) -> RustFile {
        RustFile {
            path: PathBuf::from("src/big.rs"),
            relative_path: PathBuf::from("src/big.rs"),
            package_name: "krate".to_string(),
            size_bytes: bytes,
            line_count: lines,
        }
    }
    fn s(fns: usize, impls: usize) -> FileStructure {
        FileStructure {
            function_count: fns,
            impl_block_count: impls,
            path: PathBuf::from("src/big.rs"),
            ..Default::default()
        }
    }

    #[test]
    fn no_finding_below_limit() {
        assert!(analyze(&f(100, 2000), &s(5, 1), &Threshold::Normal, &mut 0).is_none());
    }
    #[test]
    fn medium_at_limit() {
        assert_eq!(
            analyze(&f(500, 20000), &s(5, 1), &Threshold::Normal, &mut 0)
                .unwrap()
                .severity,
            Severity::Medium
        );
    }
    #[test]
    fn high_at_2x() {
        let r = analyze(&f(1000, 40000), &s(30, 5), &Threshold::Normal, &mut 0).unwrap();
        assert_eq!(r.severity, Severity::High);
        assert!(r.evidence.iter().any(|e| e.contains("function_count: 30")));
        assert!(r.evidence.iter().any(|e| e.contains("impl_block_count: 5")));
    }
    #[test]
    fn critical_at_4x() {
        assert_eq!(
            analyze(&f(2000, 80000), &s(50, 10), &Threshold::Normal, &mut 0)
                .unwrap()
                .severity,
            Severity::Critical
        );
    }
    #[test]
    fn token_estimate() {
        assert_eq!(estimate_tokens(40000), 10000);
    }
    #[test]
    fn counter_increments() {
        let mut c = 0;
        analyze(&f(600, 24000), &s(5, 1), &Threshold::Normal, &mut c);
        analyze(&f(600, 24000), &s(5, 1), &Threshold::Normal, &mut c);
        assert_eq!(c, 2);
    }
}
