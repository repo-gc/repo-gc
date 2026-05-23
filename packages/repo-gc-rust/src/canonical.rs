use serde::Deserialize;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Deserialization shims — borrow from the compile-time `include_str!` string
// so all string fields are naturally `&'static str`.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CanonicalConfig<'a> {
    #[serde(borrow)]
    finding_kinds: Vec<FindingKindEntry<'a>>,
    #[serde(borrow)]
    severities: Vec<SeverityEntry<'a>>,
    thresholds: ThresholdConfig,
}

#[derive(Debug, Deserialize)]
struct FindingKindEntry<'a> {
    id: &'a str,
    label: &'a str,
    llm_code: &'a str,
    category: &'a str,
}

#[derive(Debug, Deserialize)]
struct SeverityEntry<'a> {
    id: &'a str,
    weight: f64,
    label: &'a str,
    llm_label: &'a str,
}

#[derive(Debug, Deserialize)]
struct ThresholdConfig {
    strict: ThresholdValues,
    normal: ThresholdValues,
    relaxed: ThresholdValues,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThresholdValues {
    pub line_count_limit: usize,
    pub fan_in_limit: usize,
    pub fan_out_limit: usize,
    pub reexport_limit: usize,
    pub branch_density_limit: usize,
    pub nesting_depth_limit: usize,
    pub type_depth_limit: usize,
    pub comment_ratio_min: f64,
    pub comment_ratio_max: f64,
    pub decorator_density_limit: f64,
    pub empty_catch_limit: usize,
    pub dangerous_pattern_limit: usize,
    pub string_comparison_limit: usize,
    pub import_domain_limit: usize,
}

// ---------------------------------------------------------------------------
// Compile-time loading
// ---------------------------------------------------------------------------

const RAW: &str = include_str!("../../../test-fixtures/finding-kinds.json");

fn config() -> &'static CanonicalConfig<'static> {
    static CONFIG: OnceLock<CanonicalConfig<'static>> = OnceLock::new();
    CONFIG.get_or_init(|| serde_json::from_str(RAW).expect("canonical finding-kinds.json is valid"))
}

// ---------------------------------------------------------------------------
// Public lookup helpers
// ---------------------------------------------------------------------------

pub fn kind_label(id: &str) -> &'static str {
    config()
        .finding_kinds
        .iter()
        .find(|k| k.id == id)
        .map(|k| k.label)
        .unwrap_or_else(|| panic!("unknown finding kind id: {id}"))
}

pub fn kind_llm_code(id: &str) -> &'static str {
    config()
        .finding_kinds
        .iter()
        .find(|k| k.id == id)
        .map(|k| k.llm_code)
        .unwrap_or_else(|| panic!("unknown finding kind id: {id}"))
}

#[allow(dead_code)]
pub fn kind_category(id: &str) -> &'static str {
    config()
        .finding_kinds
        .iter()
        .find(|k| k.id == id)
        .map(|k| k.category)
        .unwrap_or_else(|| panic!("unknown finding kind id: {id}"))
}

pub fn severity_weight(id: &str) -> f64 {
    config()
        .severities
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.weight)
        .unwrap_or_else(|| panic!("unknown severity id: {id}"))
}

pub fn severity_label(id: &str) -> &'static str {
    config()
        .severities
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.label)
        .unwrap_or_else(|| panic!("unknown severity id: {id}"))
}

pub fn severity_llm_label(id: &str) -> &'static str {
    config()
        .severities
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.llm_label)
        .unwrap_or_else(|| panic!("unknown severity id: {id}"))
}

pub fn threshold_values(level: &str) -> &'static ThresholdValues {
    match level {
        "strict" => &config().thresholds.strict,
        "normal" => &config().thresholds.normal,
        "relaxed" => &config().thresholds.relaxed,
        _ => panic!("unknown threshold level: {level}"),
    }
}

#[allow(dead_code)]
pub fn kind_ids() -> &'static [&'static str] {
    static IDS: OnceLock<Vec<&'static str>> = OnceLock::new();
    IDS.get_or_init(|| config().finding_kinds.iter().map(|k| k.id).collect())
}

#[allow(dead_code)]
pub fn severity_ids() -> &'static [&'static str] {
    static IDS: OnceLock<Vec<&'static str>> = OnceLock::new();
    IDS.get_or_init(|| config().severities.iter().map(|s| s.id).collect())
}
