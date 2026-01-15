// Tue Jan 13 2026 - Alex

use crate::config::Config;
use crate::memory::{MemoryReader, MemoryError};
use crate::pattern::PatternMatcher;
use crate::xref::XRefAnalyzer;
use crate::symbol::SymbolResolver;
use crate::analysis::Analyzer;
use crate::finders::result::FinderResults;
use crate::engine::scheduler::TaskScheduler;
use crate::engine::pipeline::Pipeline;
use crate::engine::task::{Task, TaskType};
use crate::engine::result::TaskResult;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct Engine {
    config: Config,
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: Arc<PatternMatcher>,
    xref_analyzer: Arc<RwLock<XRefAnalyzer>>,
    symbol_resolver: Arc<RwLock<SymbolResolver>>,
    analyzer: Arc<Analyzer>,
    scheduler: TaskScheduler,
    results: Arc<RwLock<FinderResults>>,
    state: EngineState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Idle,
    Initializing,
    Running,
    Paused,
    Completed,
    Failed,
}

impl Engine {
    pub fn new(config: Config, reader: Arc<dyn MemoryReader>) -> Self {
        let pattern_matcher = Arc::new(PatternMatcher::new(reader.clone()));
        let xref_analyzer = Arc::new(RwLock::new(XRefAnalyzer::new(reader.clone())));
        let symbol_resolver = Arc::new(RwLock::new(SymbolResolver::new(reader.clone())));
        let analyzer = Arc::new(Analyzer::new(reader.clone()));
        let scheduler = TaskScheduler::new(config.thread_count);

        Self {
            config,
            reader,
            pattern_matcher,
            xref_analyzer,
            symbol_resolver,
            analyzer,
            scheduler,
            results: Arc::new(RwLock::new(FinderResults::new())),
            state: EngineState::Idle,
        }
    }

    pub fn initialize(&mut self) -> Result<(), EngineError> {
        self.state = EngineState::Initializing;

        {
            let mut resolver = self.symbol_resolver.write();
            resolver.load_symbols()
                .map_err(|e| EngineError::SymbolLoadFailed(e.to_string()))?;
        }

        {
            let mut xref = self.xref_analyzer.write();
            xref.initialize()
                .map_err(|e| EngineError::XRefInitFailed(e.to_string()))?;
        }

        self.state = EngineState::Idle;
        Ok(())
    }

    pub fn run(&mut self) -> Result<FinderResults, EngineError> {
        self.state = EngineState::Running;

        let pipeline = self.create_pipeline();

        for stage in pipeline.stages() {
            if self.state == EngineState::Paused {
                return Err(EngineError::Paused);
            }

            let tasks = stage.generate_tasks();

            for task in tasks {
                self.scheduler.submit(task);
            }

            let stage_results = self.scheduler.wait_for_completion();

            for result in stage_results {
                self.process_result(result)?;
            }
        }

        self.state = EngineState::Completed;

        let results = self.results.read().clone();
        Ok(results)
    }

    pub fn pause(&mut self) {
        if self.state == EngineState::Running {
            self.state = EngineState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == EngineState::Paused {
            self.state = EngineState::Running;
        }
    }

    pub fn stop(&mut self) {
        self.scheduler.stop();
        self.state = EngineState::Idle;
    }

    pub fn state(&self) -> EngineState {
        self.state
    }

    pub fn results(&self) -> FinderResults {
        self.results.read().clone()
    }

    fn create_pipeline(&self) -> Pipeline {
        let mut pipeline = Pipeline::new();

        pipeline.add_stage_with_tasks("Symbol Resolution", vec![
            Task::new(TaskType::ResolveSymbols),
        ]);

        pipeline.add_stage_with_tasks("Pattern Scanning", vec![
            Task::new(TaskType::ScanLuaApi),
            Task::new(TaskType::ScanRobloxFunctions),
            Task::new(TaskType::ScanBytecode),
        ]);

        pipeline.add_stage_with_tasks("XRef Analysis", vec![
            Task::new(TaskType::BuildCallGraph),
            Task::new(TaskType::AnalyzeXRefs),
        ]);

        pipeline.add_stage_with_tasks("Structure Analysis", vec![
            Task::new(TaskType::AnalyzeLuaState),
            Task::new(TaskType::AnalyzeExtraSpace),
            Task::new(TaskType::AnalyzeClosure),
            Task::new(TaskType::AnalyzeProto),
        ]);

        pipeline.add_stage_with_tasks("Class Analysis", vec![
            Task::new(TaskType::AnalyzeClasses),
            Task::new(TaskType::AnalyzeProperties),
            Task::new(TaskType::AnalyzeMethods),
        ]);

        pipeline.add_stage_with_tasks("Constant Analysis", vec![
            Task::new(TaskType::FindConstants),
        ]);

        pipeline.add_stage_with_tasks("Validation", vec![
            Task::new(TaskType::ValidateResults),
        ]);

        pipeline
    }

    fn process_result(&self, result: TaskResult) -> Result<(), EngineError> {
        match result {
            TaskResult::Success(findings) => {
                let mut results = self.results.write();
                results.merge(findings);
                Ok(())
            }
            TaskResult::Error(e) => {
                Err(EngineError::TaskFailed(e))
            }
            TaskResult::Skipped(reason) => {
                Ok(())
            }
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }

    pub fn pattern_matcher(&self) -> &Arc<PatternMatcher> {
        &self.pattern_matcher
    }

    pub fn xref_analyzer(&self) -> &Arc<RwLock<XRefAnalyzer>> {
        &self.xref_analyzer
    }

    pub fn symbol_resolver(&self) -> &Arc<RwLock<SymbolResolver>> {
        &self.symbol_resolver
    }

    pub fn analyzer(&self) -> &Arc<Analyzer> {
        &self.analyzer
    }
}

#[derive(Debug)]
pub enum EngineError {
    MemoryError(MemoryError),
    SymbolLoadFailed(String),
    XRefInitFailed(String),
    TaskFailed(String),
    Paused,
    InvalidState(String),
}

impl From<MemoryError> for EngineError {
    fn from(e: MemoryError) -> Self {
        EngineError::MemoryError(e)
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::MemoryError(e) => write!(f, "Memory error: {}", e),
            EngineError::SymbolLoadFailed(e) => write!(f, "Symbol load failed: {}", e),
            EngineError::XRefInitFailed(e) => write!(f, "XRef initialization failed: {}", e),
            EngineError::TaskFailed(e) => write!(f, "Task failed: {}", e),
            EngineError::Paused => write!(f, "Engine is paused"),
            EngineError::InvalidState(e) => write!(f, "Invalid state: {}", e),
        }
    }
}

impl std::error::Error for EngineError {}
