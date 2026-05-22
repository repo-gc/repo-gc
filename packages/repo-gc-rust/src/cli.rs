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
    pub fn line_count_limit(&self) -> usize {
        match self {
            Threshold::Strict => 300,
            Threshold::Normal => 500,
            Threshold::Relaxed => 1000,
        }
    }
    pub fn fan_in_limit(&self) -> usize {
        match self {
            Threshold::Strict => 5,
            Threshold::Normal => 10,
            Threshold::Relaxed => 20,
        }
    }
    pub fn fan_out_limit(&self) -> usize {
        match self {
            Threshold::Strict => 10,
            Threshold::Normal => 15,
            Threshold::Relaxed => 25,
        }
    }
    pub fn reexport_limit(&self) -> usize {
        match self {
            Threshold::Strict => 5,
            Threshold::Normal => 10,
            Threshold::Relaxed => 20,
        }
    }
    pub fn branch_density_limit(&self) -> usize {
        match self { Threshold::Strict => 8, Threshold::Normal => 12, Threshold::Relaxed => 18 }
    }
    pub fn nesting_depth_limit(&self) -> usize {
        match self { Threshold::Strict => 4, Threshold::Normal => 6, Threshold::Relaxed => 8 }
    }
    pub fn type_depth_limit(&self) -> usize {
        match self { Threshold::Strict => 3, Threshold::Normal => 4, Threshold::Relaxed => 5 }
    }
    pub fn comment_ratio_min(&self) -> f64 {
        match self { Threshold::Strict => 0.03, Threshold::Normal => 0.03, Threshold::Relaxed => 0.01 }
    }
    pub fn comment_ratio_max(&self) -> f64 {
        match self { Threshold::Strict => 0.20, Threshold::Normal => 0.30, Threshold::Relaxed => 0.50 }
    }
    pub fn decorator_density_limit(&self) -> f64 {
        match self { Threshold::Strict => 0.33, Threshold::Normal => 0.50, Threshold::Relaxed => 0.75 }
    }
    pub fn empty_catch_limit(&self) -> usize {
        match self { Threshold::Strict => 1, Threshold::Normal => 2, Threshold::Relaxed => 3 }
    }
    pub fn dangerous_pattern_limit(&self) -> usize {
        match self { Threshold::Strict => 3, Threshold::Normal => 5, Threshold::Relaxed => 10 }
    }
    pub fn string_comparison_limit(&self) -> usize {
        match self { Threshold::Strict => 5, Threshold::Normal => 10, Threshold::Relaxed => 15 }
    }
    pub fn import_domain_limit(&self) -> usize {
        match self { Threshold::Strict => 8, Threshold::Normal => 10, Threshold::Relaxed => 14 }
    }
}
