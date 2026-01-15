// Tue Jan 13 2026 - Alex

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target_process: Option<String>,
    pub target_binary: Option<PathBuf>,
    pub output_file: PathBuf,
    pub scan_memory: bool,
    pub scan_binary: bool,
    pub enable_pattern_scanning: bool,
    pub enable_symbol_matching: bool,
    pub enable_xref_analysis: bool,
    pub enable_heuristic_analysis: bool,
    pub max_threads: usize,
    pub pattern_confidence_threshold: f64,
    pub symbol_match_confidence: f64,
    pub xref_depth_limit: usize,
    pub structure_alignment: usize,
    pub enable_verbose_output: bool,
    pub enable_progress_bars: bool,
    pub timeout_seconds: u64,
    pub memory_scan_regions: Vec<ConfigMemoryRegion>,
    pub binary_sections: Vec<String>,
    pub skip_validation: bool,
    pub parallel_discovery: bool,
    pub cache_symbols: bool,
    pub cache_patterns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMemoryRegion {
    pub start: u64,
    pub end: u64,
    pub protection: u32,
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_process: None,
            target_binary: None,
            output_file: PathBuf::from("offsets.json"),
            scan_memory: true,
            scan_binary: true,
            enable_pattern_scanning: true,
            enable_symbol_matching: true,
            enable_xref_analysis: true,
            enable_heuristic_analysis: true,
            max_threads: num_cpus::get(),
            pattern_confidence_threshold: 0.85,
            symbol_match_confidence: 0.90,
            xref_depth_limit: 10,
            structure_alignment: 8,
            enable_verbose_output: false,
            enable_progress_bars: true,
            timeout_seconds: 300,
            memory_scan_regions: Vec::new(),
            binary_sections: vec!["__TEXT".to_string(), "__DATA".to_string()],
            skip_validation: false,
            parallel_discovery: true,
            cache_symbols: true,
            cache_patterns: true,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target_process(mut self, process: String) -> Self {
        self.target_process = Some(process);
        self
    }

    pub fn with_target_binary(mut self, binary: PathBuf) -> Self {
        self.target_binary = Some(binary);
        self
    }

    pub fn with_output_file(mut self, output: PathBuf) -> Self {
        self.output_file = output;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.target_process.is_none() && self.target_binary.is_none() {
            return Err("Either target_process or target_binary must be set".to_string());
        }
        if !self.scan_memory && !self.scan_binary {
            return Err("At least one of scan_memory or scan_binary must be enabled".to_string());
        }
        if self.max_threads == 0 {
            return Err("max_threads must be greater than 0".to_string());
        }
        if self.pattern_confidence_threshold < 0.0 || self.pattern_confidence_threshold > 1.0 {
            return Err("pattern_confidence_threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }
}
