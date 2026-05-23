use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "repo-gc", version, about = "Find patterns that waste tokens, confuse AI agents, and break AI-assisted edits")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan the repository for AI-context-wasting patterns
    Scan {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "text")] format: OutputFormat,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long, default_value = "false")] include_tests: bool,
        #[arg(long)] no_color: bool,
        #[arg(long, default_value = "rust")] lang: String,
    },
    /// Generate an AI Context Efficiency report
    Report {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "text")] format: OutputFormat,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long)] include_tests: bool,
        #[arg(long, default_value = "rust")] lang: String,
    },
    /// Emit findings as JSON (for CI integration)
    Json {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long)] include_tests: bool,
        #[arg(long, default_value = "rust")] lang: String,
    },
    /// Explain why a file degrades AI context efficiency
    Explain {
        path: PathBuf,
        #[arg(long, default_value = ".")] root: PathBuf,
    },
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Md,
    /// Token-optimized TSV for LLM consumption
    Llm,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum Threshold {
    Strict,
    Normal,
    Relaxed,
}

impl Threshold {
    fn level(&self) -> &'static str {
        match self {
            Threshold::Strict => "strict",
            Threshold::Normal => "normal",
            Threshold::Relaxed => "relaxed",
        }
    }

    pub fn line_count_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).line_count_limit
    }
    pub fn fan_in_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).fan_in_limit
    }
    pub fn fan_out_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).fan_out_limit
    }
    pub fn reexport_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).reexport_limit
    }
    pub fn branch_density_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).branch_density_limit
    }
    pub fn nesting_depth_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).nesting_depth_limit
    }
    pub fn type_depth_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).type_depth_limit
    }
    pub fn comment_ratio_min(&self) -> f64 {
        crate::canonical::threshold_values(self.level()).comment_ratio_min
    }
    pub fn comment_ratio_max(&self) -> f64 {
        crate::canonical::threshold_values(self.level()).comment_ratio_max
    }
    pub fn decorator_density_limit(&self) -> f64 {
        crate::canonical::threshold_values(self.level()).decorator_density_limit
    }
    pub fn empty_catch_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).empty_catch_limit
    }
    pub fn dangerous_pattern_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).dangerous_pattern_limit
    }
    pub fn string_comparison_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).string_comparison_limit
    }
    pub fn import_domain_limit(&self) -> usize {
        crate::canonical::threshold_values(self.level()).import_domain_limit
    }
}
