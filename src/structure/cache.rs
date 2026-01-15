// Tue Jan 13 2026 - Alex

use crate::structure::StructureLayout;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct StructureCache {
    cache: RwLock<HashMap<String, StructureLayout>>,
}

impl StructureCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &str) -> Option<StructureLayout> {
        self.cache.read().get(name).cloned()
    }

    pub fn insert(&self, layout: StructureLayout) {
        self.cache.write().insert(layout.name().to_string(), layout);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }
}

impl Default for StructureCache {
    fn default() -> Self {
        Self::new()
    }
}
