// Tue Jan 13 2026 - Alex

pub mod disassembler;
pub mod block;
pub mod cfg;
pub mod function;
pub mod dataflow;
pub mod pattern;
pub mod heuristics;
pub mod string;
pub mod signature;
pub mod cross_reference;

pub use disassembler::{Disassembler, DisassembledInstruction};
pub use block::BasicBlock;
pub use cfg::ControlFlowGraph;
pub use function::{FunctionAnalyzer, AnalyzedFunction};
pub use dataflow::DataFlowAnalyzer;
pub use pattern::PatternRecognizer;
pub use heuristics::HeuristicAnalyzer;
pub use string::StringAnalyzer;
pub use signature::SignatureAnalyzer;
pub use cross_reference::CrossReferenceAnalyzer;

use crate::memory::{MemoryReader, MemoryError, Address};
use std::sync::Arc;

pub struct Analyzer {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
    function_analyzer: FunctionAnalyzer,
    heuristic_analyzer: HeuristicAnalyzer,
    string_analyzer: StringAnalyzer,
    pattern_recognizer: PatternRecognizer,
}

impl Analyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let disassembler = Arc::new(Disassembler::new(reader.clone()));

        Self {
            reader: reader.clone(),
            disassembler: disassembler.clone(),
            function_analyzer: FunctionAnalyzer::new(reader.clone(), disassembler.clone()),
            heuristic_analyzer: HeuristicAnalyzer::new(reader.clone(), disassembler.clone()),
            string_analyzer: StringAnalyzer::new(reader.clone()),
            pattern_recognizer: PatternRecognizer::new(),
        }
    }

    pub fn analyze_function(&self, addr: Address) -> Result<AnalyzedFunction, MemoryError> {
        self.function_analyzer.analyze(addr)
    }

    pub fn is_function_entry(&self, addr: Address) -> Result<heuristics::HeuristicResult, MemoryError> {
        self.heuristic_analyzer.is_function_entry(addr)
    }

    pub fn find_strings(&self) -> Result<Vec<string::FoundString>, MemoryError> {
        let regions = self.reader.get_regions()?;
        let mut all_strings = Vec::new();

        for region in &regions {
            if region.protection.is_readable() {
                let strings = self.string_analyzer.find_strings_in_region(region)?;
                all_strings.extend(strings);
            }
        }

        Ok(all_strings)
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }

    pub fn disassembler(&self) -> &Arc<Disassembler> {
        &self.disassembler
    }
}
