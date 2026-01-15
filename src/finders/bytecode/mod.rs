// Tue Jan 13 2026 - Alex

pub mod opcode_lookup;
pub mod decoder;
pub mod analyzer;

pub use opcode_lookup::{OpcodeLookupFinder, find_opcode_lookup};
use crate::memory::{Address, MemoryReader};
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub fn find_all_bytecode_functions(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<FinderResult> {
    let mut results = Vec::new();

    if let Some(r) = find_opcode_lookup(reader.clone(), start, end) {
        results.push(r);
    }

    results
}
