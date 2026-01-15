// Tue Jan 13 2026 - Alex

use crate::config::Config;
use crate::memory::process::ProcessMemory;
use crate::memory::binary::BinaryMemory;
use crate::pattern::PatternMatcher;
use crate::xref::XRefAnalyzer;
use crate::orchestration::discovery::DiscoveryManager;
use crate::orchestration::scheduler::DiscoveryScheduler;
use crate::orchestration::collector::ResultCollector;
use crate::orchestration::aggregator::ResultAggregator;
use crate::orchestration::finalizer::OutputFinalizer;
use crate::finders::result::FinderResults;
use crate::output::manager::OutputManager;
use crate::ui::progress::ProgressManager;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

pub struct DiscoveryCoordinator {
    config: Arc<Config>,
    process_memory: Arc<ProcessMemory>,
    binary_memory: Arc<BinaryMemory>,
    pattern_matcher: Arc<PatternMatcher>,
    xref_analyzer: Arc<XRefAnalyzer>,
    discovery_manager: Arc<RwLock<DiscoveryManager>>,
    scheduler: Arc<DiscoveryScheduler>,
    collector: Arc<RwLock<ResultCollector>>,
    aggregator: Arc<ResultAggregator>,
    finalizer: Arc<OutputFinalizer>,
    progress_manager: Arc<ProgressBarManager>,
}

impl DiscoveryCoordinator {
    pub fn new(
        config: Arc<Config>,
        process_memory: Arc<ProcessMemory>,
        binary_memory: Arc<BinaryMemory>,
        pattern_matcher: Arc<PatternMatcher>,
        xref_analyzer: Arc<XRefAnalyzer>,
        progress_manager: Arc<ProgressBarManager>,
    ) -> Self {
        let discovery_manager = Arc::new(RwLock::new(DiscoveryManager::new(
            pattern_matcher.clone(),
            xref_analyzer.clone(),
        )));

        let scheduler = Arc::new(DiscoveryScheduler::new(config.thread_count as usize));
        let collector = Arc::new(RwLock::new(ResultCollector::new()));
        let aggregator = Arc::new(ResultAggregator::new());
        let finalizer = Arc::new(OutputFinalizer::new());

        Self {
            config,
            process_memory,
            binary_memory,
            pattern_matcher,
            xref_analyzer,
            discovery_manager,
            scheduler,
            collector,
            aggregator,
            finalizer,
            progress_manager,
        }
    }

    pub fn run_discovery(&self) -> Result<OutputManager> {
        let main_progress = self.progress_manager.create_main_progress(
            "Discovering offsets".to_string(),
            6,
        );

        main_progress.set_message("Phase 1: Pattern scanning");
        let pattern_results = self.run_pattern_scanning()?;
        main_progress.inc(1);

        main_progress.set_message("Phase 2: Symbol analysis");
        let symbol_results = self.run_symbol_analysis()?;
        main_progress.inc(1);

        main_progress.set_message("Phase 3: XRef analysis");
        let xref_results = self.run_xref_analysis()?;
        main_progress.inc(1);

        main_progress.set_message("Phase 4: Structure analysis");
        let structure_results = self.run_structure_analysis()?;
        main_progress.inc(1);

        main_progress.set_message("Phase 5: Heuristic analysis");
        let heuristic_results = self.run_heuristic_analysis()?;
        main_progress.inc(1);

        main_progress.set_message("Phase 6: Aggregating results");
        let aggregated = self.aggregator.aggregate(vec![
            pattern_results,
            symbol_results,
            xref_results,
            structure_results,
            heuristic_results,
        ]);
        main_progress.inc(1);

        main_progress.finish_with_message("Discovery complete");

        let output = self.finalizer.finalize(aggregated, &self.config);
        Ok(output)
    }

    fn run_pattern_scanning(&self) -> Result<FinderResults> {
        let mut results = FinderResults::new();

        let discovery = self.discovery_manager.read();
        let pattern_results = discovery.discover_patterns()?;
        results.merge(pattern_results);

        Ok(results)
    }

    fn run_symbol_analysis(&self) -> Result<FinderResults> {
        let mut results = FinderResults::new();

        let discovery = self.discovery_manager.read();
        let symbol_results = discovery.discover_symbols()?;
        results.merge(symbol_results);

        Ok(results)
    }

    fn run_xref_analysis(&self) -> Result<FinderResults> {
        let mut results = FinderResults::new();

        let discovery = self.discovery_manager.read();
        let xref_results = discovery.discover_xrefs()?;
        results.merge(xref_results);

        Ok(results)
    }

    fn run_structure_analysis(&self) -> Result<FinderResults> {
        let mut results = FinderResults::new();

        let discovery = self.discovery_manager.read();
        let structure_results = discovery.discover_structures()?;
        results.merge(structure_results);

        Ok(results)
    }

    fn run_heuristic_analysis(&self) -> Result<FinderResults> {
        let mut results = FinderResults::new();

        let discovery = self.discovery_manager.read();
        let heuristic_results = discovery.discover_heuristics()?;
        results.merge(heuristic_results);

        Ok(results)
    }

    pub fn get_statistics(&self) -> DiscoveryStatistics {
        let collector = self.collector.read();
        DiscoveryStatistics {
            patterns_found: collector.pattern_count(),
            symbols_resolved: collector.symbol_count(),
            xrefs_analyzed: collector.xref_count(),
            structures_found: collector.structure_count(),
            total_offsets: collector.total_count(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryStatistics {
    pub patterns_found: usize,
    pub symbols_resolved: usize,
    pub xrefs_analyzed: usize,
    pub structures_found: usize,
    pub total_offsets: usize,
}

impl DiscoveryStatistics {
    pub fn summary(&self) -> String {
        format!(
            "Patterns: {}, Symbols: {}, XRefs: {}, Structures: {}, Total: {}",
            self.patterns_found,
            self.symbols_resolved,
            self.xrefs_analyzed,
            self.structures_found,
            self.total_offsets
        )
    }
}
