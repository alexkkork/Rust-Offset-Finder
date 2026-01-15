// Tue Jan 15 2026 - Alex

#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unreachable_patterns)]
#![allow(unused_must_use)]
#![allow(ambiguous_glob_reexports)]
#![allow(unpredictable_function_pointer_comparisons)]

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
pub mod scripting;
pub mod diff;

pub use config::Config;
pub use memory::MemoryScanner;
pub use pattern::PatternMatcher;
pub use xref::XRefAnalyzer;
pub use structure::StructureAnalyzer;
pub use output::OutputManager;
pub use engine::core::Engine;
pub use luau::VmAnalyzer;
pub use orchestration::DiscoveryCoordinator;
pub use validation::OffsetValidator;
pub use scripting::{ScriptEngine, ScriptResult};