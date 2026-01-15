// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::symbol::SymbolInfo;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct SymbolCache {
    cache: RwLock<HashMap<String, SymbolInfo>>,
    address_cache: RwLock<HashMap<u64, SymbolInfo>>,
}

impl SymbolCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            address_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &str) -> Option<SymbolInfo> {
        self.cache.read().get(name).cloned()
    }

    pub fn get_by_address(&self, address: Address) -> Option<SymbolInfo> {
        self.address_cache.read().get(&address.as_u64()).cloned()
    }

    pub fn insert(&self, symbol: SymbolInfo) {
        self.cache.write().insert(symbol.name().to_string(), symbol.clone());
        self.address_cache.write().insert(symbol.address().as_u64(), symbol);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
        self.address_cache.write().clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }
}

impl Default for SymbolCache {
    fn default() -> Self {
        Self::new()
    }
}
