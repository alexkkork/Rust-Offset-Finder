// Wed Jan 15 2026 - Alex

use crate::memory::Address;
use crate::analysis::disasm::DecodedInstruction;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct DisassemblyCache {
    cache: Arc<RwLock<HashMap<u64, DecodedInstruction>>>,
    max_size: usize,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl DisassemblyCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_size))),
            max_size,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    pub fn get(&self, addr: Address) -> Option<DecodedInstruction> {
        let cache = self.cache.read().unwrap();
        if let Some(instr) = cache.get(&addr.as_u64()) {
            let mut hits = self.hits.write().unwrap();
            *hits += 1;
            Some(instr.clone())
        } else {
            let mut misses = self.misses.write().unwrap();
            *misses += 1;
            None
        }
    }

    pub fn insert(&self, addr: Address, instr: DecodedInstruction) {
        let mut cache = self.cache.write().unwrap();

        if cache.len() >= self.max_size {
            self.evict(&mut cache);
        }

        cache.insert(addr.as_u64(), instr);
    }

    pub fn contains(&self, addr: Address) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(&addr.as_u64())
    }

    pub fn remove(&self, addr: Address) -> Option<DecodedInstruction> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(&addr.as_u64())
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        let mut hits = self.hits.write().unwrap();
        let mut misses = self.misses.write().unwrap();
        *hits = 0;
        *misses = 0;
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.read().unwrap();
        let misses = *self.misses.read().unwrap();
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let hits = *self.hits.read().unwrap();
        let misses = *self.misses.read().unwrap();

        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hits,
            misses,
            hit_rate: self.hit_rate(),
        }
    }

    fn evict(&self, cache: &mut HashMap<u64, DecodedInstruction>) {
        let to_remove = cache.len() / 4;
        let keys: Vec<u64> = cache.keys().take(to_remove).copied().collect();
        for key in keys {
            cache.remove(&key);
        }
    }

    pub fn get_range(&self, start: Address, end: Address) -> Vec<DecodedInstruction> {
        let cache = self.cache.read().unwrap();
        let mut result = Vec::new();

        let mut addr = start.as_u64();
        while addr < end.as_u64() {
            if let Some(instr) = cache.get(&addr) {
                result.push(instr.clone());
                addr += instr.size as u64;
            } else {
                addr += 4;
            }
        }

        result
    }

    pub fn insert_batch(&self, instructions: &[DecodedInstruction]) {
        let mut cache = self.cache.write().unwrap();

        for instr in instructions {
            if cache.len() >= self.max_size {
                self.evict(&mut cache);
            }
            cache.insert(instr.address.as_u64(), instr.clone());
        }
    }
}

impl Clone for DisassemblyCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            max_size: self.max_size,
            hits: self.hits.clone(),
            misses: self.misses.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn format(&self) -> String {
        format!(
            "Cache: {}/{} entries ({:.1}% full), {:.1}% hit rate ({} hits, {} misses)",
            self.size,
            self.max_size,
            self.size as f64 / self.max_size as f64 * 100.0,
            self.hit_rate * 100.0,
            self.hits,
            self.misses
        )
    }
}

pub struct FunctionCache {
    cache: Arc<RwLock<HashMap<u64, Vec<DecodedInstruction>>>>,
    max_functions: usize,
}

impl FunctionCache {
    pub fn new(max_functions: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_functions))),
            max_functions,
        }
    }

    pub fn get(&self, entry: Address) -> Option<Vec<DecodedInstruction>> {
        let cache = self.cache.read().unwrap();
        cache.get(&entry.as_u64()).cloned()
    }

    pub fn insert(&self, entry: Address, instructions: Vec<DecodedInstruction>) {
        let mut cache = self.cache.write().unwrap();

        if cache.len() >= self.max_functions {
            let to_remove = cache.len() / 4;
            let keys: Vec<u64> = cache.keys().take(to_remove).copied().collect();
            for key in keys {
                cache.remove(&key);
            }
        }

        cache.insert(entry.as_u64(), instructions);
    }

    pub fn contains(&self, entry: Address) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(&entry.as_u64())
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for FunctionCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            max_functions: self.max_functions,
        }
    }
}
