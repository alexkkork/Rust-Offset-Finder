// Tue Jan 13 2026 - Alex

pub mod resolver;
pub mod demangle;
pub mod export;
pub mod import;

pub use resolver::{SymbolResolver, Symbol, SymbolType, SymbolCache};

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

pub fn create_symbol_resolver(reader: Arc<dyn MemoryReader>) -> SymbolResolver {
    SymbolResolver::new(reader)
}

pub fn create_symbol_cache(reader: Arc<dyn MemoryReader>) -> SymbolCache {
    SymbolCache::new(reader)
}
