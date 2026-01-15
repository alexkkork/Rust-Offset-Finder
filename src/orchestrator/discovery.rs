// Tue Jan 13 2026 - Alex

use crate::memory::{MemoryReader, MemoryError, Address, MemoryRegion};
use crate::pattern::PatternMatcher;
use crate::xref::XRefAnalyzer;
use crate::symbol::SymbolResolver;
use crate::analysis::Analyzer;
use crate::finders::result::FinderResults;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use rayon::prelude::*;

pub struct DiscoveryOrchestrator {
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: Arc<PatternMatcher>,
    xref_analyzer: Arc<RwLock<XRefAnalyzer>>,
    symbol_resolver: Arc<RwLock<SymbolResolver>>,
    analyzer: Arc<Analyzer>,
    discovery_strategies: Vec<Box<dyn DiscoveryStrategy + Send + Sync>>,
}

impl DiscoveryOrchestrator {
    pub fn new(
        reader: Arc<dyn MemoryReader>,
        pattern_matcher: Arc<PatternMatcher>,
        xref_analyzer: Arc<RwLock<XRefAnalyzer>>,
        symbol_resolver: Arc<RwLock<SymbolResolver>>,
        analyzer: Arc<Analyzer>,
    ) -> Self {
        Self {
            reader,
            pattern_matcher,
            xref_analyzer,
            symbol_resolver,
            analyzer,
            discovery_strategies: Vec::new(),
        }
    }

    pub fn add_strategy<S: DiscoveryStrategy + Send + Sync + 'static>(&mut self, strategy: S) {
        self.discovery_strategies.push(Box::new(strategy));
    }

    pub fn discover_all(&self) -> Result<FinderResults, DiscoveryError> {
        let mut combined_results = FinderResults::new();

        let symbol_results = self.discover_from_symbols()?;
        combined_results.merge(symbol_results);

        let pattern_results = self.discover_from_patterns()?;
        combined_results.merge(pattern_results);

        let xref_results = self.discover_from_xrefs()?;
        combined_results.merge(xref_results);

        let heuristic_results = self.discover_from_heuristics()?;
        combined_results.merge(heuristic_results);

        Ok(combined_results)
    }

    pub fn discover_from_symbols(&self) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();
        let resolver = self.symbol_resolver.read();

        let function_prefixes = vec![
            "lua_", "luau_", "luaL_", "luaB_", "luaC_", "luaD_", "luaE_",
            "luaF_", "luaG_", "luaH_", "luaI_", "luaK_", "luaM_", "luaO_",
            "luaS_", "luaT_", "luaU_", "luaV_", "luaX_", "luaZ_",
        ];

        for prefix in function_prefixes {
            let symbols = resolver.find_by_prefix(prefix);
            for symbol in symbols {
                if symbol.is_function() {
                    results.functions.insert(symbol.name.clone(), symbol.address);
                }
            }
        }

        let roblox_keywords = vec![
            "Roblox", "Instance", "Script", "DataModel", "Workspace",
            "Players", "ReplicatedStorage", "ServerStorage",
        ];

        for keyword in roblox_keywords {
            let symbols = resolver.find_by_contains(keyword);
            for symbol in symbols {
                if symbol.is_function() {
                    results.functions.insert(symbol.name.clone(), symbol.address);
                } else {
                    results.classes.insert(symbol.name.clone(), symbol.address);
                }
            }
        }

        Ok(results)
    }

    pub fn discover_from_patterns(&self) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();
        let regions = self.reader.get_regions()
            .map_err(|e| DiscoveryError::MemoryError(e))?;

        let executable_regions: Vec<_> = regions.iter()
            .filter(|r| r.protection.is_executable())
            .collect();

        let patterns = self.get_discovery_patterns();

        for (name, pattern, mask) in patterns {
            match self.pattern_matcher.find_first(&pattern, &mask, &regions) {
                Ok(Some(addr)) => {
                    results.functions.insert(name, addr);
                }
                Ok(None) => {}
                Err(e) => {
                    log::warn!("Pattern search failed for {}: {}", name, e);
                }
            }
        }

        Ok(results)
    }

    pub fn discover_from_xrefs(&self) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();

        let strings_to_find = vec![
            "lua_pushstring",
            "lua_getfield",
            "lua_setfield",
            "lua_pcall",
            "print",
            "error",
            "assert",
        ];

        Ok(results)
    }

    pub fn discover_from_heuristics(&self) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();

        let regions = self.reader.get_regions()
            .map_err(|e| DiscoveryError::MemoryError(e))?;

        for region in &regions {
            if !region.protection.is_executable() {
                continue;
            }

            let potential_functions = self.find_function_prologues(region)?;
            for (addr, confidence) in potential_functions {
                if confidence > 0.7 {
                    results.functions.insert(
                        format!("sub_{:x}", addr.as_u64()),
                        addr,
                    );
                }
            }
        }

        Ok(results)
    }

    fn get_discovery_patterns(&self) -> Vec<(String, String, String)> {
        vec![
            ("lua_pushstring".to_string(), 
             "FD 7B BF A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("lua_getfield".to_string(),
             "FD 7B BE A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("lua_setfield".to_string(),
             "FD 7B BD A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("lua_rawget".to_string(),
             "FD 7B BC A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("lua_rawset".to_string(),
             "FD 7B BB A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("luau_load".to_string(),
             "FD 7B BA A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
            
            ("rbx_pushinstance".to_string(),
             "FD 7B B9 A9 FD 03 00 91".to_string(),
             "FF FF FF FF FF FF FF FF".to_string()),
        ]
    }

    fn find_function_prologues(&self, region: &MemoryRegion) -> Result<Vec<(Address, f64)>, DiscoveryError> {
        let mut functions = Vec::new();

        let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)
            .map_err(|e| DiscoveryError::MemoryError(e))?;

        for offset in (0..data.len().saturating_sub(8)).step_by(4) {
            let word = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
            ]);

            let is_stp_x29_x30 = (word & 0xFFC07FFF) == 0xA9007BFD;
            let is_sub_sp = (word & 0xFF0003FF) == 0xD10003FF;
            let is_stp_general = (word & 0xFE000000) == 0xA9000000;

            if is_stp_x29_x30 {
                let addr = Address::new(region.range.start.as_u64() + offset as u64);
                functions.push((addr, 0.9));
            } else if is_sub_sp || is_stp_general {
                let addr = Address::new(region.range.start.as_u64() + offset as u64);
                functions.push((addr, 0.6));
            }
        }

        Ok(functions)
    }

    pub fn run_custom_strategy(&self, strategy_name: &str) -> Result<FinderResults, DiscoveryError> {
        for strategy in &self.discovery_strategies {
            if strategy.name() == strategy_name {
                return strategy.discover(
                    self.reader.clone(),
                    self.pattern_matcher.clone(),
                );
            }
        }

        Err(DiscoveryError::StrategyNotFound(strategy_name.to_string()))
    }
}

#[derive(Debug)]
pub enum DiscoveryError {
    MemoryError(MemoryError),
    PatternError(String),
    SymbolError(String),
    XRefError(String),
    StrategyNotFound(String),
}

impl From<MemoryError> for DiscoveryError {
    fn from(e: MemoryError) -> Self {
        DiscoveryError::MemoryError(e)
    }
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::MemoryError(e) => write!(f, "Memory error: {}", e),
            DiscoveryError::PatternError(e) => write!(f, "Pattern error: {}", e),
            DiscoveryError::SymbolError(e) => write!(f, "Symbol error: {}", e),
            DiscoveryError::XRefError(e) => write!(f, "XRef error: {}", e),
            DiscoveryError::StrategyNotFound(e) => write!(f, "Strategy not found: {}", e),
        }
    }
}

impl std::error::Error for DiscoveryError {}

pub trait DiscoveryStrategy {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn discover(
        &self,
        reader: Arc<dyn MemoryReader>,
        pattern_matcher: Arc<PatternMatcher>,
    ) -> Result<FinderResults, DiscoveryError>;
}

pub struct LuaApiDiscoveryStrategy;

impl DiscoveryStrategy for LuaApiDiscoveryStrategy {
    fn name(&self) -> &str {
        "LuaApiDiscovery"
    }

    fn description(&self) -> &str {
        "Discovers Lua API functions through pattern matching and symbol resolution"
    }

    fn discover(
        &self,
        reader: Arc<dyn MemoryReader>,
        pattern_matcher: Arc<PatternMatcher>,
    ) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();

        let regions = reader.get_regions()
            .map_err(|e| DiscoveryError::MemoryError(e))?;

        Ok(results)
    }
}

pub struct RobloxFunctionDiscoveryStrategy;

impl DiscoveryStrategy for RobloxFunctionDiscoveryStrategy {
    fn name(&self) -> &str {
        "RobloxFunctionDiscovery"
    }

    fn description(&self) -> &str {
        "Discovers Roblox-specific functions"
    }

    fn discover(
        &self,
        reader: Arc<dyn MemoryReader>,
        pattern_matcher: Arc<PatternMatcher>,
    ) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();

        Ok(results)
    }
}

pub struct StructureDiscoveryStrategy;

impl DiscoveryStrategy for StructureDiscoveryStrategy {
    fn name(&self) -> &str {
        "StructureDiscovery"
    }

    fn description(&self) -> &str {
        "Discovers structure layouts and offsets"
    }

    fn discover(
        &self,
        reader: Arc<dyn MemoryReader>,
        pattern_matcher: Arc<PatternMatcher>,
    ) -> Result<FinderResults, DiscoveryError> {
        let mut results = FinderResults::new();

        Ok(results)
    }
}
