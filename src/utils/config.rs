// Tue Jan 13 2026 - Alex

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub general: GeneralConfig,
    pub scanning: ScanningConfig,
    pub output: OutputConfig,
    pub patterns: PatternConfig,
    pub finders: FindersConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub verbose: bool,
    pub quiet: bool,
    pub threads: usize,
    pub timeout_seconds: u64,
    pub cache_enabled: bool,
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    pub enable_patterns: bool,
    pub enable_symbols: bool,
    pub enable_xrefs: bool,
    pub enable_heuristics: bool,
    pub confidence_threshold: f64,
    pub max_scan_depth: usize,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub pretty_print: bool,
    pub include_metadata: bool,
    pub include_statistics: bool,
    pub backup_enabled: bool,
    pub backup_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    pub custom_patterns: HashMap<String, String>,
    pub disabled_patterns: Vec<String>,
    pub pattern_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindersConfig {
    pub enabled_finders: Vec<String>,
    pub disabled_finders: Vec<String>,
    pub finder_options: HashMap<String, HashMap<String, String>>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            scanning: ScanningConfig::default(),
            output: OutputConfig::default(),
            patterns: PatternConfig::default(),
            finders: FindersConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            quiet: false,
            threads: num_cpus::get(),
            timeout_seconds: 300,
            cache_enabled: true,
            cache_dir: None,
        }
    }
}

impl Default for ScanningConfig {
    fn default() -> Self {
        Self {
            enable_patterns: true,
            enable_symbols: true,
            enable_xrefs: true,
            enable_heuristics: true,
            confidence_threshold: 0.85,
            max_scan_depth: 10,
            sections: vec!["__TEXT".to_string(), "__DATA".to_string()],
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            pretty_print: true,
            include_metadata: true,
            include_statistics: true,
            backup_enabled: true,
            backup_count: 3,
        }
    }
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            custom_patterns: HashMap::new(),
            disabled_patterns: Vec::new(),
            pattern_cache_size: 1000,
        }
    }
}

impl Default for FindersConfig {
    fn default() -> Self {
        Self {
            enabled_finders: Vec::new(),
            disabled_finders: Vec::new(),
            finder_options: HashMap::new(),
        }
    }
}

impl ConfigFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::NotFound(path.to_path_buf()));
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext.to_lowercase().as_str() {
            "json" => serde_json::from_str(&contents)
                .map_err(|e| ConfigError::ParseError(e.to_string())),
            "toml" => Err(ConfigError::ParseError("TOML support not compiled in".to_string())),
            _ => Err(ConfigError::UnsupportedFormat(ext.to_string())),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("json");

        let contents = match ext.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(self)
                .map_err(|e| ConfigError::SerializeError(e.to_string()))?,
            _ => return Err(ConfigError::UnsupportedFormat(ext.to_string())),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        fs::write(path, contents)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn merge(&mut self, other: &ConfigFile) {
        if other.general.verbose {
            self.general.verbose = true;
        }
        if other.general.quiet {
            self.general.quiet = true;
        }

        if !other.patterns.custom_patterns.is_empty() {
            self.patterns.custom_patterns.extend(other.patterns.custom_patterns.clone());
        }

        self.patterns.disabled_patterns.extend(other.patterns.disabled_patterns.clone());
        self.finders.disabled_finders.extend(other.finders.disabled_finders.clone());
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.general.threads == 0 {
            return Err(ConfigError::ValidationError("threads must be > 0".to_string()));
        }

        if self.scanning.confidence_threshold < 0.0 || self.scanning.confidence_threshold > 1.0 {
            return Err(ConfigError::ValidationError(
                "confidence_threshold must be between 0.0 and 1.0".to_string()
            ));
        }

        Ok(())
    }

    pub fn get_default_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("roblox-offset-generator")
            .join("config.json")
    }

    pub fn is_finder_enabled(&self, name: &str) -> bool {
        if self.finders.disabled_finders.contains(&name.to_string()) {
            return false;
        }

        if self.finders.enabled_finders.is_empty() {
            return true;
        }

        self.finders.enabled_finders.contains(&name.to_string())
    }

    pub fn is_pattern_enabled(&self, name: &str) -> bool {
        !self.patterns.disabled_patterns.contains(&name.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    NotFound(PathBuf),
    IoError(String),
    ParseError(String),
    SerializeError(String),
    UnsupportedFormat(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound(path) => write!(f, "Config file not found: {:?}", path),
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            ConfigError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
            ConfigError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ConfigFile, ConfigError> {
    ConfigFile::load(path)
}

pub fn save_config<P: AsRef<Path>>(config: &ConfigFile, path: P) -> Result<(), ConfigError> {
    config.save(path)
}

pub fn default_config() -> ConfigFile {
    ConfigFile::default()
}
