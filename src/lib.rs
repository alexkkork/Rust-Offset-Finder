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
pub mod engine;
pub mod luau;
pub mod orchestration;
pub mod validation;

pub use config::Config;
pub use memory::MemoryScanner;
pub use pattern::PatternMatcher;
pub use xref::XRefAnalyzer;
pub use structure::StructureAnalyzer;
pub use output::OutputManager;
pub use engine::core::EngineCore;
pub use luau::LuauVm;
pub use orchestration::DiscoveryCoordinator;
pub use validation::OffsetValidator;
