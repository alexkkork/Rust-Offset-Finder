// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::pattern::{MatchResult, PatternMask};
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct PatternCache {
    cache: RwLock<HashMap<(Address, usize), Vec<MatchResult>>>,
    max_size: usize,
}

impl PatternCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
        }
    }

    pub fn get(&self, start: Address, size: usize) -> Option<Vec<MatchResult>> {
        self.cache.read().get(&(start, size)).cloned()
    }

    pub fn insert(&self, start: Address, size: usize, results: Vec<MatchResult>) {
        let mut cache = self.cache.write();
        if cache.len() >= self.max_size {
            cache.clear();
        }
        cache.insert((start, size), results);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }
}
