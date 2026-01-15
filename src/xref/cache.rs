// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::xref::CallGraph;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct XRefCache {
    cache: RwLock<HashMap<u64, CallGraph>>,
}

impl XRefCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, address: Address) -> Option<CallGraph> {
        self.cache.read().get(&address.as_u64()).cloned()
    }

    pub fn insert(&self, address: Address, graph: CallGraph) {
        self.cache.write().insert(address.as_u64(), graph);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }
}

impl Default for XRefCache {
    fn default() -> Self {
        Self::new()
    }
}
