// Tue Jan 13 2026 - Alex

pub mod finder;
pub mod types;

pub use finder::ConstantFinder;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::ConstantResult;
use std::sync::Arc;

pub fn find_all_constants(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<ConstantResult> {
    let finder = ConstantFinder::new(reader);
    finder.find_all(start, end)
}
