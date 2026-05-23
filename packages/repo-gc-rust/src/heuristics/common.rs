//! Shared helpers used across heuristic modules.

use crate::types::Severity;

/// Multiplicative severity scaling: `>= limit*3` → Critical, `>= limit*2` → High, else Medium.
pub fn severity_scale(value: f64, limit: f64) -> Severity {
    if value >= limit * 3.0 {
        Severity::Critical
    } else if value >= limit * 2.0 {
        Severity::High
    } else {
        Severity::Medium
    }
}

/// Additive severity scaling for depth-based heuristics: `>= limit+high_offset` → Critical,
/// `>= limit+med_offset` → High, else Medium.
pub fn severity_scale_offset(value: usize, limit: usize, high_offset: usize, crit_offset: usize) -> Severity {
    if value >= limit + crit_offset {
        Severity::Critical
    } else if value >= limit + high_offset {
        Severity::High
    } else {
        Severity::Medium
    }
}