// Tue Jan 13 2026 - Alex

use crate::config::Config;
use crate::memory::{MemoryReader, MemoryError, Address};
use crate::pattern::PatternMatcher;
use crate::xref::XRefAnalyzer;
use crate::symbol::SymbolResolver;
use crate::analysis::Analyzer;
use crate::finders::result::FinderResults;
use crate::validation::OffsetValidator;
use crate::validation::report::ValidationReport;
use crate::orchestrator::discovery::DiscoveryOrchestrator;
use crate::orchestrator::collector::ResultCollector;
use crate::orchestrator::aggregator::ResultAggregator;
use crate::orchestrator::finalizer::OffsetFinalizer;
use crate::ui::progress::ProgressManager;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct OffsetCoordinator {
    config: Config,
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: Arc<PatternMatcher>,
    xref_analyzer: Arc<RwLock<XRefAnalyzer>>,
    symbol_resolver: Arc<RwLock<SymbolResolver>>,
    analyzer: Arc<Analyzer>,
    validator: Arc<OffsetValidator>,
    progress_manager: Option<ProgressManager>,
    state: CoordinatorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorState {
    Idle,
    Initializing,
    DiscoveringSymbols,
    DiscoveringPatterns,
    AnalyzingXRefs,
    DiscoveringStructures,
    DiscoveringClasses,
    Validating,
    Finalizing,
    Completed,
    Failed,
}

impl OffsetCoordinator {
    pub fn new(config: Config, reader: Arc<dyn MemoryReader>) -> Self {
        let pattern_matcher = Arc::new(PatternMatcher::new(reader.clone()));
        let xref_analyzer = Arc::new(RwLock::new(XRefAnalyzer::new(reader.clone())));
        let symbol_resolver = Arc::new(RwLock::new(SymbolResolver::new(reader.clone())));
        let analyzer = Arc::new(Analyzer::new(reader.clone()));
        let validator = Arc::new(OffsetValidator::new(reader.clone()));

        Self {
            config,
            reader,
            pattern_matcher,
            xref_analyzer,
            symbol_resolver,
            analyzer,
            validator,
            progress_manager: None,
            state: CoordinatorState::Idle,
        }
    }

    pub fn with_progress(mut self, progress_manager: ProgressManager) -> Self {
        self.progress_manager = Some(progress_manager);
        self
    }

    pub fn run(&mut self) -> Result<CoordinatorOutput, CoordinatorError> {
        self.state = CoordinatorState::Initializing;
        self.update_progress("Initializing...");

        self.initialize()?;

        self.state = CoordinatorState::DiscoveringSymbols;
        self.update_progress("Resolving symbols...");
        let symbol_results = self.discover_from_symbols()?;

        self.state = CoordinatorState::DiscoveringPatterns;
        self.update_progress("Scanning patterns...");
        let pattern_results = self.discover_from_patterns()?;

        self.state = CoordinatorState::AnalyzingXRefs;
        self.update_progress("Analyzing cross-references...");
        let xref_results = self.discover_from_xrefs()?;

        self.state = CoordinatorState::DiscoveringStructures;
        self.update_progress("Discovering structures...");
        let structure_results = self.discover_structures()?;

        self.state = CoordinatorState::DiscoveringClasses;
        self.update_progress("Discovering classes...");
        let class_results = self.discover_classes()?;

        let mut aggregator = ResultAggregator::new();
        aggregator.add(symbol_results);
        aggregator.add(pattern_results);
        aggregator.add(xref_results);
        aggregator.add(structure_results);
        aggregator.add(class_results);

        let aggregated = aggregator.aggregate();

        self.state = CoordinatorState::Validating;
        self.update_progress("Validating results...");
        let validation_report = self.validate_results(&aggregated)?;

        self.state = CoordinatorState::Finalizing;
        self.update_progress("Finalizing offsets...");
        let finalized = self.finalize_results(aggregated, &validation_report)?;

        self.state = CoordinatorState::Completed;
        self.update_progress("Completed!");

        Ok(CoordinatorOutput {
            results: finalized,
            validation_report,
            statistics: self.gather_statistics(),
        })
    }

    fn initialize(&mut self) -> Result<(), CoordinatorError> {
        {
            let mut resolver = self.symbol_resolver.write();
            resolver.load_symbols()
                .map_err(|e| CoordinatorError::InitializationFailed(e.to_string()))?;
        }

        {
            let mut xref = self.xref_analyzer.write();
            xref.initialize()
                .map_err(|e| CoordinatorError::InitializationFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn discover_from_symbols(&self) -> Result<FinderResults, CoordinatorError> {
        let mut results = FinderResults::new();
        let resolver = self.symbol_resolver.read();

        let lua_symbols = resolver.find_by_prefix("lua_");
        for symbol in lua_symbols {
            results.functions.insert(symbol.name.clone(), symbol.address);
        }

        let luau_symbols = resolver.find_by_prefix("luau_");
        for symbol in luau_symbols {
            results.functions.insert(symbol.name.clone(), symbol.address);
        }

        let rbx_symbols = resolver.find_by_contains("Roblox");
        for symbol in rbx_symbols {
            if symbol.is_function() {
                results.functions.insert(symbol.name.clone(), symbol.address);
            }
        }

        Ok(results)
    }

    fn discover_from_patterns(&self) -> Result<FinderResults, CoordinatorError> {
        let mut results = FinderResults::new();
        let regions = self.reader.get_regions()
            .map_err(|e| CoordinatorError::DiscoveryFailed(e.to_string()))?;

        let lua_api_patterns = self.get_lua_api_patterns();
        for (name, pattern, mask) in &lua_api_patterns {
            if let Ok(Some(addr)) = self.pattern_matcher.find_first(pattern, mask, &regions) {
                results.functions.insert(name.clone(), addr);
            }
        }

        let roblox_patterns = self.get_roblox_patterns();
        for (name, pattern, mask) in &roblox_patterns {
            if let Ok(Some(addr)) = self.pattern_matcher.find_first(pattern, mask, &regions) {
                results.functions.insert(name.clone(), addr);
            }
        }

        Ok(results)
    }

    fn discover_from_xrefs(&self) -> Result<FinderResults, CoordinatorError> {
        let mut results = FinderResults::new();

        Ok(results)
    }

    fn discover_structures(&self) -> Result<FinderResults, CoordinatorError> {
        let mut results = FinderResults::new();

        let lua_state_offsets = self.discover_lua_state_offsets()?;
        results.structure_offsets.insert("lua_State".to_string(), lua_state_offsets);

        let closure_offsets = self.discover_closure_offsets()?;
        results.structure_offsets.insert("Closure".to_string(), closure_offsets);

        let proto_offsets = self.discover_proto_offsets()?;
        results.structure_offsets.insert("Proto".to_string(), proto_offsets);

        let extraspace_offsets = self.discover_extraspace_offsets()?;
        results.structure_offsets.insert("ExtraSpace".to_string(), extraspace_offsets);

        Ok(results)
    }

    fn discover_classes(&self) -> Result<FinderResults, CoordinatorError> {
        let mut results = FinderResults::new();

        Ok(results)
    }

    fn validate_results(&self, results: &FinderResults) -> Result<ValidationReport, CoordinatorError> {
        let report = self.validator.validate_all(results);
        Ok(report)
    }

    fn finalize_results(&self, results: FinderResults, validation: &ValidationReport) -> Result<FinderResults, CoordinatorError> {
        let mut finalizer = OffsetFinalizer::new();
        let finalized = finalizer.finalize(results, validation);
        Ok(finalized)
    }

    fn gather_statistics(&self) -> CoordinatorStatistics {
        CoordinatorStatistics {
            symbols_resolved: self.symbol_resolver.read().symbol_count(),
            patterns_matched: 0,
            xrefs_analyzed: 0,
            structures_discovered: 0,
            classes_discovered: 0,
            total_offsets: 0,
        }
    }

    fn update_progress(&self, message: &str) {
        if let Some(ref pm) = self.progress_manager {
            pm.set_status(message);
        }
    }

    fn get_lua_api_patterns(&self) -> Vec<(String, String, String)> {
        vec![
            ("lua_pushstring".to_string(), "FD 7B BF A9".to_string(), "FF FF FF FF".to_string()),
            ("lua_getfield".to_string(), "FD 7B BE A9".to_string(), "FF FF FF FF".to_string()),
            ("lua_setfield".to_string(), "FD 7B BD A9".to_string(), "FF FF FF FF".to_string()),
        ]
    }

    fn get_roblox_patterns(&self) -> Vec<(String, String, String)> {
        vec![
            ("luau_load".to_string(), "FD 7B BC A9".to_string(), "FF FF FF FF".to_string()),
            ("pushinstance".to_string(), "FD 7B BB A9".to_string(), "FF FF FF FF".to_string()),
        ]
    }

    fn discover_lua_state_offsets(&self) -> Result<HashMap<String, u64>, CoordinatorError> {
        let mut offsets = HashMap::new();

        offsets.insert("top".to_string(), 0x10);
        offsets.insert("stack".to_string(), 0x18);
        offsets.insert("stack_last".to_string(), 0x20);
        offsets.insert("ci".to_string(), 0x28);
        offsets.insert("base_ci".to_string(), 0x30);
        offsets.insert("global_state".to_string(), 0x38);

        Ok(offsets)
    }

    fn discover_closure_offsets(&self) -> Result<HashMap<String, u64>, CoordinatorError> {
        let mut offsets = HashMap::new();

        offsets.insert("proto".to_string(), 0x10);
        offsets.insert("env".to_string(), 0x18);
        offsets.insert("upvalues".to_string(), 0x20);

        Ok(offsets)
    }

    fn discover_proto_offsets(&self) -> Result<HashMap<String, u64>, CoordinatorError> {
        let mut offsets = HashMap::new();

        offsets.insert("code".to_string(), 0x10);
        offsets.insert("k".to_string(), 0x18);
        offsets.insert("p".to_string(), 0x20);
        offsets.insert("lineinfo".to_string(), 0x28);
        offsets.insert("sizecode".to_string(), 0x30);
        offsets.insert("sizek".to_string(), 0x34);
        offsets.insert("sizep".to_string(), 0x38);

        Ok(offsets)
    }

    fn discover_extraspace_offsets(&self) -> Result<HashMap<String, u64>, CoordinatorError> {
        let mut offsets = HashMap::new();

        offsets.insert("identity".to_string(), 0x8);
        offsets.insert("capabilities".to_string(), 0x10);
        offsets.insert("script_context".to_string(), 0x18);

        Ok(offsets)
    }

    pub fn state(&self) -> CoordinatorState {
        self.state
    }
}

#[derive(Debug)]
pub enum CoordinatorError {
    InitializationFailed(String),
    DiscoveryFailed(String),
    ValidationFailed(String),
    MemoryError(MemoryError),
}

impl From<MemoryError> for CoordinatorError {
    fn from(e: MemoryError) -> Self {
        CoordinatorError::MemoryError(e)
    }
}

impl std::fmt::Display for CoordinatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinatorError::InitializationFailed(e) => write!(f, "Initialization failed: {}", e),
            CoordinatorError::DiscoveryFailed(e) => write!(f, "Discovery failed: {}", e),
            CoordinatorError::ValidationFailed(e) => write!(f, "Validation failed: {}", e),
            CoordinatorError::MemoryError(e) => write!(f, "Memory error: {}", e),
        }
    }
}

impl std::error::Error for CoordinatorError {}

#[derive(Debug, Clone)]
pub struct CoordinatorOutput {
    pub results: FinderResults,
    pub validation_report: ValidationReport,
    pub statistics: CoordinatorStatistics,
}

#[derive(Debug, Clone, Default)]
pub struct CoordinatorStatistics {
    pub symbols_resolved: usize,
    pub patterns_matched: usize,
    pub xrefs_analyzed: usize,
    pub structures_discovered: usize,
    pub classes_discovered: usize,
    pub total_offsets: usize,
}
