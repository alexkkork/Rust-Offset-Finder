// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::symbol::{SymbolInfo, SymbolError, SymbolKind};
use std::collections::HashMap;

pub struct SymbolTable {
    symbols: HashMap<String, SymbolInfo>,
    by_address: HashMap<u64, SymbolInfo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            by_address: HashMap::new(),
        }
    }

    pub fn add(&mut self, symbol: SymbolInfo) {
        self.symbols.insert(symbol.name().to_string(), symbol.clone());
        self.by_address.insert(symbol.address().as_u64(), symbol);
    }

    pub fn find(&self, name: &str) -> Result<Option<SymbolInfo>, SymbolError> {
        Ok(self.symbols.get(name).cloned())
    }

    pub fn find_by_address(&self, address: Address) -> Result<Option<SymbolInfo>, SymbolError> {
        Ok(self.by_address.get(&address.as_u64()).cloned())
    }

    pub fn get_functions(&self) -> Result<Vec<SymbolInfo>, SymbolError> {
        Ok(self.symbols.values()
            .filter(|s| s.is_function())
            .cloned()
            .collect())
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
