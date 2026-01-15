// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryRegion, MemoryError};
use crate::pattern::Pattern;
use std::sync::Arc;

pub struct PatternMatcher {
    reader: Arc<dyn MemoryReader>,
    chunk_size: usize,
    parallel: bool,
}

impl PatternMatcher {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            chunk_size: 0x10000,
            parallel: true,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn set_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn find_first(&self, pattern: &str, mask: &str, regions: &[MemoryRegion]) -> Result<Option<Address>, MemoryError> {
        let pat = self.parse_pattern(pattern, mask);

        for region in regions {
            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    if let Some(match_offset) = pat.find_in(&data) {
                        return Ok(Some(addr + match_offset as u64));
                    }
                }

                offset += self.chunk_size - pat.len();
            }
        }

        Ok(None)
    }

    pub fn find_all(&self, pattern: &str, mask: &str, regions: &[MemoryRegion]) -> Result<Vec<Address>, MemoryError> {
        let pat = self.parse_pattern(pattern, mask);
        let mut results = Vec::new();

        for region in regions {
            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    for match_offset in pat.find_all_in(&data) {
                        results.push(addr + match_offset as u64);
                    }
                }

                offset += self.chunk_size - pat.len();
            }
        }

        Ok(results)
    }

    pub fn find_pattern_in_range(&self, pattern_bytes: &[u8], start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();
        let pattern = Pattern::from_bytes(pattern_bytes);
        let size = (end.as_u64() - start.as_u64()) as usize;

        let mut offset = 0;
        while offset < size {
            let read_size = (size - offset).min(self.chunk_size);
            let addr = start + offset as u64;

            if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                for match_offset in pattern.find_all_in(&data) {
                    results.push(addr + match_offset as u64);
                }
            }

            let overlap = pattern.len().saturating_sub(1);
            offset += self.chunk_size.saturating_sub(overlap);
        }

        Ok(results)
    }

    pub fn find_pattern(&self, pattern: &Pattern, regions: &[MemoryRegion]) -> Result<Option<Address>, MemoryError> {
        for region in regions {
            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    if let Some(match_offset) = pattern.find_in(&data) {
                        return Ok(Some(addr + match_offset as u64));
                    }
                }

                offset += self.chunk_size - pattern.len();
            }
        }

        Ok(None)
    }

    pub fn find_all_patterns(&self, pattern: &Pattern, regions: &[MemoryRegion]) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();

        for region in regions {
            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    for match_offset in pattern.find_all_in(&data) {
                        results.push(addr + match_offset as u64);
                    }
                }

                offset += self.chunk_size - pattern.len();
            }
        }

        Ok(results)
    }

    fn parse_pattern(&self, pattern: &str, mask: &str) -> Pattern {
        let pattern_bytes: Vec<u8> = pattern.split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        let mask_bytes: Vec<bool> = mask.split_whitespace()
            .map(|s| s != "??" && s != "?")
            .collect();

        if pattern_bytes.len() != mask_bytes.len() {
            return Pattern::from_hex(pattern);
        }

        Pattern::new(pattern_bytes, mask_bytes)
    }
}

pub struct MultiPatternMatcher {
    patterns: Vec<Pattern>,
    reader: Arc<dyn MemoryReader>,
    chunk_size: usize,
}

impl MultiPatternMatcher {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            patterns: Vec::new(),
            reader,
            chunk_size: 0x10000,
        }
    }

    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }

    pub fn add_patterns(&mut self, patterns: impl IntoIterator<Item = Pattern>) {
        self.patterns.extend(patterns);
    }

    pub fn find_all(&self, regions: &[MemoryRegion]) -> Result<Vec<(usize, Address)>, MemoryError> {
        let mut results = Vec::new();

        for region in regions {
            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let max_pattern_len = self.patterns.iter()
                .map(|p| p.len())
                .max()
                .unwrap_or(0);

            let mut offset = 0;
            while offset < size {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    for (pattern_idx, pattern) in self.patterns.iter().enumerate() {
                        for match_offset in pattern.find_all_in(&data) {
                            results.push((pattern_idx, addr + match_offset as u64));
                        }
                    }
                }

                offset += self.chunk_size - max_pattern_len;
            }
        }

        Ok(results)
    }

    pub fn find_first_of_each(&self, regions: &[MemoryRegion]) -> Result<Vec<Option<Address>>, MemoryError> {
        let mut results: Vec<Option<Address>> = vec![None; self.patterns.len()];
        let mut found_count = 0;

        for region in regions {
            if found_count >= self.patterns.len() {
                break;
            }

            if !region.protection().is_readable() {
                continue;
            }

            let start = region.range().start();
            let size = region.range().size() as usize;

            let max_pattern_len = self.patterns.iter()
                .map(|p| p.len())
                .max()
                .unwrap_or(0);

            let mut offset = 0;
            while offset < size && found_count < self.patterns.len() {
                let read_size = (size - offset).min(self.chunk_size);
                let addr = start + offset as u64;

                if let Ok(data) = self.reader.read_bytes(addr, read_size) {
                    for (pattern_idx, pattern) in self.patterns.iter().enumerate() {
                        if results[pattern_idx].is_none() {
                            if let Some(match_offset) = pattern.find_in(&data) {
                                results[pattern_idx] = Some(addr + match_offset as u64);
                                found_count += 1;
                            }
                        }
                    }
                }

                offset += self.chunk_size - max_pattern_len;
            }
        }

        Ok(results)
    }
}
