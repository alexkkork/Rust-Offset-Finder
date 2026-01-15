// Tue Jan 13 2026 - Alex

pub mod pattern;
pub mod matcher;
pub mod compiler;
pub mod scanner;
pub mod database;
pub mod arm64;

pub use pattern::Pattern;
pub use matcher::PatternMatcher;
pub use scanner::PatternScanner;
pub use database::PatternDatabase;

use crate::memory::{Address, MemoryReader, MemoryRegion};

pub fn scan_for_pattern(
    reader: &dyn MemoryReader,
    pattern: &Pattern,
    regions: &[MemoryRegion],
) -> Vec<Address> {
    let scanner = PatternScanner::new();
    scanner.scan(reader, pattern, regions)
}

pub fn scan_for_patterns(
    reader: &dyn MemoryReader,
    patterns: &[Pattern],
    regions: &[MemoryRegion],
) -> Vec<(usize, Address)> {
    let scanner = PatternScanner::new();
    scanner.scan_multiple(reader, patterns, regions)
}
