use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "repo-gc", version, about = "AI-era Rust repository hygiene analyzer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan the repository and print diagnostics
    Scan {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "text")] format: OutputFormat,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long, default_value = "false")] include_tests: bool,
        #[arg(long)] no_color: bool,
    },
    /// Generate a summary report
    Report {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "text")] format: OutputFormat,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long)] include_tests: bool,
    },
    /// Emit diagnostics as JSON
    Json {
        #[arg(long, default_value = ".")] path: PathBuf,
        #[arg(long, value_enum, default_value = "normal")] threshold: Threshold,
        #[arg(long)] include_tests: bool,
    },
    /// Explain why a file is considered problematic
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
}
