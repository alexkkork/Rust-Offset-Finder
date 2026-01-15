// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::symbol::{Symbol, SymbolType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExportedSymbol {
    pub name: String,
    pub address: Address,
    pub ordinal: Option<u32>,
    pub flags: ExportFlags,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExportFlags {
    pub weak: bool,
    pub reexport: bool,
    pub stub_and_resolver: bool,
}

pub struct ExportTable {
    exports: HashMap<String, ExportedSymbol>,
    by_address: HashMap<u64, String>,
    by_ordinal: HashMap<u32, String>,
}

impl ExportTable {
    pub fn new() -> Self {
        Self {
            exports: HashMap::new(),
            by_address: HashMap::new(),
            by_ordinal: HashMap::new(),
        }
    }

    pub fn add(&mut self, export: ExportedSymbol) {
        self.by_address.insert(export.address.as_u64(), export.name.clone());

        if let Some(ordinal) = export.ordinal {
            self.by_ordinal.insert(ordinal, export.name.clone());
        }

        self.exports.insert(export.name.clone(), export);
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ExportedSymbol> {
        self.exports.get(name)
    }

    pub fn get_by_address(&self, addr: Address) -> Option<&ExportedSymbol> {
        self.by_address.get(&addr.as_u64())
            .and_then(|name| self.exports.get(name))
    }

    pub fn get_by_ordinal(&self, ordinal: u32) -> Option<&ExportedSymbol> {
        self.by_ordinal.get(&ordinal)
            .and_then(|name| self.exports.get(name))
    }

    pub fn iter(&self) -> impl Iterator<Item = &ExportedSymbol> {
        self.exports.values()
    }

    pub fn len(&self) -> usize {
        self.exports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.exports.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.exports.contains_key(name)
    }

    pub fn to_symbols(&self) -> Vec<Symbol> {
        self.exports.values()
            .map(|e| Symbol {
                name: e.name.clone(),
                address: e.address,
                size: None,
                symbol_type: SymbolType::External,
                demangled_name: None,
            })
            .collect()
    }

    pub fn find_prefix(&self, prefix: &str) -> Vec<&ExportedSymbol> {
        self.exports.iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(_, export)| export)
            .collect()
    }

    pub fn find_suffix(&self, suffix: &str) -> Vec<&ExportedSymbol> {
        self.exports.iter()
            .filter(|(name, _)| name.ends_with(suffix))
            .map(|(_, export)| export)
            .collect()
    }

    pub fn find_contains(&self, substring: &str) -> Vec<&ExportedSymbol> {
        self.exports.iter()
            .filter(|(name, _)| name.contains(substring))
            .map(|(_, export)| export)
            .collect()
    }
}

impl Default for ExportTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportedSymbol {
    pub fn new(name: String, address: Address) -> Self {
        Self {
            name,
            address,
            ordinal: None,
            flags: ExportFlags::default(),
        }
    }

    pub fn with_ordinal(mut self, ordinal: u32) -> Self {
        self.ordinal = Some(ordinal);
        self
    }

    pub fn with_flags(mut self, flags: ExportFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn is_weak(&self) -> bool {
        self.flags.weak
    }

    pub fn is_reexport(&self) -> bool {
        self.flags.reexport
    }
}
