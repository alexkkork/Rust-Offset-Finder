// Wed Jan 15 2026 - Alex

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "roblox-offset-generator")]
#[command(author = "Alex")]
#[command(version = "1.0.0")]
#[command(about = "ARM64 Roblox offset finder for macOS", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long, global = true, default_value = "info")]
    pub log_level: String,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub no_color: bool,

    #[arg(long, global = true)]
    pub json_output: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Generate(GenerateArgs),
    Diff(DiffArgs),
    Stats(StatsArgs),
    Validate(ValidateArgs),
    Dump(DumpArgs),
}

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[arg(short, long)]
    pub binary: Option<PathBuf>,

    #[arg(short, long)]
    pub process: Option<String>,

    #[arg(short, long, default_value = "offsets.json")]
    pub output: PathBuf,

    #[arg(long, default_value = "8")]
    pub threads: usize,

    #[arg(long, default_value = "0.7")]
    pub confidence_threshold: f64,

    #[arg(long)]
    pub pattern_db: Option<PathBuf>,

    #[arg(long)]
    pub no_symbols: bool,

    #[arg(long)]
    pub no_xrefs: bool,

    #[arg(long)]
    pub no_heuristics: bool,

    #[arg(long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct DiffArgs {
    #[arg(short, long)]
    pub old: PathBuf,

    #[arg(short, long)]
    pub new: PathBuf,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub show_unchanged: bool,
}

#[derive(Parser, Debug)]
pub struct StatsArgs {
    #[arg(short, long)]
    pub input: PathBuf,

    #[arg(long)]
    pub detailed: bool,

    #[arg(long)]
    pub by_category: bool,
}

#[derive(Parser, Debug)]
pub struct ValidateArgs {
    #[arg(short, long)]
    pub offsets: PathBuf,

    #[arg(short, long)]
    pub binary: Option<PathBuf>,

    #[arg(short, long)]
    pub process: Option<String>,

    #[arg(long)]
    pub strict: bool,
}

#[derive(Parser, Debug)]
pub struct DumpArgs {
    #[arg(short, long)]
    pub binary: Option<PathBuf>,

    #[arg(short, long)]
    pub process: Option<String>,

    #[arg(short, long)]
    pub address: String,

    #[arg(short, long, default_value = "256")]
    pub size: usize,

    #[arg(long)]
    pub disassemble: bool,
}

impl GenerateArgs {
    pub fn validate(&self) -> Result<(), String> {
        if self.binary.is_none() && self.process.is_none() {
            return Err("Either --binary or --process must be specified".to_string());
        }
        if self.binary.is_some() && self.process.is_some() {
            return Err("Cannot specify both --binary and --process".to_string());
        }
        if self.threads == 0 {
            return Err("Thread count must be at least 1".to_string());
        }
        if self.confidence_threshold < 0.0 || self.confidence_threshold > 1.0 {
            return Err("Confidence threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }
}

impl DiffArgs {
    pub fn validate(&self) -> Result<(), String> {
        if !self.old.exists() {
            return Err(format!("Old file does not exist: {:?}", self.old));
        }
        if !self.new.exists() {
            return Err(format!("New file does not exist: {:?}", self.new));
        }
        Ok(())
    }
}

impl ValidateArgs {
    pub fn validate(&self) -> Result<(), String> {
        if !self.offsets.exists() {
            return Err(format!("Offsets file does not exist: {:?}", self.offsets));
        }
        if self.binary.is_none() && self.process.is_none() {
            return Err("Either --binary or --process must be specified".to_string());
        }
        Ok(())
    }
}
