// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryRegion};
use crate::pattern::Pattern;
use rayon::prelude::*;
use std::sync::Arc;

pub struct PatternScanner {
    chunk_size: usize,
    use_parallel: bool,
    skip_unreadable: bool,
}

impl PatternScanner {
    pub fn new() -> Self {
        Self {
            chunk_size: 0x10000,
            use_parallel: true,
            skip_unreadable: true,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn use_parallel(mut self, parallel: bool) -> Self {
        self.use_parallel = parallel;
        self
    }

    pub fn skip_unreadable(mut self, skip: bool) -> Self {
        self.skip_unreadable = skip;
        self
    }

    pub fn scan(&self, reader: &dyn MemoryReader, pattern: &Pattern, regions: &[MemoryRegion]) -> Vec<Address> {
        let filtered_regions: Vec<_> = if self.skip_unreadable {
            regions.iter()
                .filter(|r| r.protection.is_readable())
                .cloned()
                .collect()
        } else {
            regions.to_vec()
        };

        if self.use_parallel {
            self.scan_parallel(reader, pattern, &filtered_regions)
        } else {
            self.scan_sequential(reader, pattern, &filtered_regions)
        }
    }

    fn scan_sequential(&self, reader: &dyn MemoryReader, pattern: &Pattern, regions: &[MemoryRegion]) -> Vec<Address> {
        let mut results = Vec::new();

        for region in regions {
            let start = region.range.start;
            let size = region.range.size as usize;

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = reader.read_bytes(addr, read_size) {
                    for match_offset in pattern.find_all_in(&data) {
                        results.push(addr + match_offset as u64);
                    }
                }

                let overlap = pattern.len().saturating_sub(1);
                offset += self.chunk_size.saturating_sub(overlap);
            }
        }

        results
    }

    fn scan_parallel(&self, reader: &dyn MemoryReader, pattern: &Pattern, regions: &[MemoryRegion]) -> Vec<Address> {
        let chunks: Vec<_> = regions.iter()
            .flat_map(|region| {
                let start = region.range.start;
                let size = region.range.size as usize;
                let overlap = pattern.len().saturating_sub(1);
                let step = self.chunk_size.saturating_sub(overlap);

                (0..size).step_by(step.max(1))
                    .map(move |offset| {
                        let read_size = (size - offset).min(self.chunk_size);
                        (start + offset as u64, read_size)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let results: Vec<Vec<Address>> = chunks.iter()
            .map(|&(addr, read_size)| {
                let mut matches = Vec::new();
                if let Ok(data) = reader.read_bytes(addr, read_size) {
                    for match_offset in pattern.find_all_in(&data) {
                        matches.push(addr + match_offset as u64);
                    }
                }
                matches
            })
            .collect();

        results.into_iter().flatten().collect()
    }

    pub fn scan_multiple(&self, reader: &dyn MemoryReader, patterns: &[Pattern], regions: &[MemoryRegion]) -> Vec<(usize, Address)> {
        let filtered_regions: Vec<_> = if self.skip_unreadable {
            regions.iter()
                .filter(|r| r.protection.is_readable())
                .cloned()
                .collect()
        } else {
            regions.to_vec()
        };

        let max_pattern_len = patterns.iter()
            .map(|p| p.len())
            .max()
            .unwrap_or(0);

        let mut results = Vec::new();

        for region in &filtered_regions {
            let start = region.range.start;
            let size = region.range.size as usize;

            let overlap = max_pattern_len.saturating_sub(1);
            let step = self.chunk_size.saturating_sub(overlap);

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = reader.read_bytes(addr, read_size) {
                    for (pattern_idx, pattern) in patterns.iter().enumerate() {
                        for match_offset in pattern.find_all_in(&data) {
                            results.push((pattern_idx, addr + match_offset as u64));
                        }
                    }
                }

                offset += step.max(1);
            }
        }

        results
    }

    pub fn scan_first(&self, reader: &dyn MemoryReader, pattern: &Pattern, regions: &[MemoryRegion]) -> Option<Address> {
        let filtered_regions: Vec<_> = if self.skip_unreadable {
            regions.iter()
                .filter(|r| r.protection.is_readable())
                .cloned()
                .collect()
        } else {
            regions.to_vec()
        };

        for region in &filtered_regions {
            let start = region.range.start;
            let size = region.range.size as usize;

            let overlap = pattern.len().saturating_sub(1);
            let step = self.chunk_size.saturating_sub(overlap);

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = reader.read_bytes(addr, read_size) {
                    if let Some(match_offset) = pattern.find_in(&data) {
                        return Some(addr + match_offset as u64);
                    }
                }

                offset += step.max(1);
            }
        }

        None
    }
}

impl Default for PatternScanner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScanProgress {
    pub bytes_scanned: u64,
    pub total_bytes: u64,
    pub matches_found: usize,
    pub regions_scanned: usize,
    pub total_regions: usize,
}

impl ScanProgress {
    pub fn new(total_bytes: u64, total_regions: usize) -> Self {
        Self {
            bytes_scanned: 0,
            total_bytes,
            matches_found: 0,
            regions_scanned: 0,
            total_regions,
        }
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            return 100.0;
        }
        (self.bytes_scanned as f64 / self.total_bytes as f64) * 100.0
    }
}
