// Tue Jan 13 2026 - Alex

pub mod config;
pub mod memory;
pub mod pattern;
pub mod symbol;
pub mod xref;
pub mod structure;
pub mod finders;
pub mod analysis;
pub mod output;
pub mod ui;
pub mod utils;

pub use config::Config;
pub use memory::MemoryScanner;
pub use pattern::PatternMatcher;
pub use symbol::SymbolResolver;
pub use xref::XRefAnalyzer;
pub use structure::StructureAnalyzer;
pub use output::OutputManager;
