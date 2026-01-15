// Tue Jan 15 2026 - Alex

pub mod resolver;
pub mod demangle;
pub mod export;
pub mod import;
pub mod dwarf;
pub mod export_formats;

pub use resolver::{SymbolResolver, Symbol, SymbolType, SymbolCache};
pub use dwarf::{DwarfParser, DwarfFunction, DwarfType, DwarfVariable, DwarfError, DwarfTag};
pub use export_formats::{SymbolExporter, ExportableSymbol, ExportFormat, ExportSymbolType, SymbolImporter};

use crate::memory::MemoryReader;
use std::sync::Arc;

pub fn create_symbol_resolver(reader: Arc<dyn MemoryReader>) -> SymbolResolver {
    SymbolResolver::new(reader)
}

pub fn create_symbol_cache(reader: Arc<dyn MemoryReader>) -> SymbolCache {
    SymbolCache::new(reader)
}
