// Tue Jan 13 2026 - Alex

pub mod finder;
pub mod analyzer;
pub mod signature;

pub use finder::MethodFinder;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::MethodResult;
use std::sync::Arc;

pub fn find_all_methods(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<MethodResult> {
    let finder = MethodFinder::new(reader);
    finder.find_all(start, end)
}
