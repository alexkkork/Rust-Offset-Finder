// Tue Jan 13 2026 - Alex

pub mod finder;
pub mod accessor;
pub mod types;

pub use finder::PropertyFinder;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::PropertyResult;
use std::sync::Arc;

pub fn find_all_properties(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<PropertyResult> {
    let finder = PropertyFinder::new(reader);
    finder.find_all(start, end)
}
