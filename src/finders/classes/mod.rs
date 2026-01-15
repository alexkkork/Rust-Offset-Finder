// Tue Jan 13 2026 - Alex

pub mod instance;
pub mod reflection;
pub mod vtable;
pub mod descriptor;
pub mod hierarchy;

pub use instance::InstanceClassFinder;
pub use reflection::ReflectionFinder;
pub use vtable::VTableAnalyzer;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::ClassResult;
use std::sync::Arc;

pub fn find_all_classes(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<ClassResult> {
    let mut results = Vec::new();

    let instance_finder = InstanceClassFinder::new(reader.clone());
    results.extend(instance_finder.find_all(start, end));

    let reflection_finder = ReflectionFinder::new(reader.clone());
    results.extend(reflection_finder.find_all(start, end));

    results
}
