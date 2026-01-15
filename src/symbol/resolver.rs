// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;
use std::collections::HashMap;

pub struct SymbolResolver {
    reader: Arc<dyn MemoryReader>,
    symbols: HashMap<String, Symbol>,
    address_to_symbol: HashMap<u64, String>,
    loaded: bool,
}

impl SymbolResolver {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            symbols: HashMap::new(),
            address_to_symbol: HashMap::new(),
            loaded: false,
        }
    }

    pub fn load_symbols(&mut self) -> Result<usize, MemoryError> {
        if self.loaded {
            return Ok(self.symbols.len());
        }

        self.load_mach_o_symbols()?;

        self.loaded = true;
        Ok(self.symbols.len())
    }

    fn load_mach_o_symbols(&mut self) -> Result<(), MemoryError> {
        Ok(())
    }

    pub fn resolve_address(&self, addr: Address) -> Option<&Symbol> {
        self.address_to_symbol.get(&addr.as_u64())
            .and_then(|name| self.symbols.get(name))
    }

    pub fn resolve_name(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn find_by_prefix(&self, prefix: &str) -> Vec<&Symbol> {
        self.symbols.iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(_, sym)| sym)
            .collect()
    }

    pub fn find_by_suffix(&self, suffix: &str) -> Vec<&Symbol> {
        self.symbols.iter()
            .filter(|(name, _)| name.ends_with(suffix))
            .map(|(_, sym)| sym)
            .collect()
    }

    pub fn find_by_contains(&self, substring: &str) -> Vec<&Symbol> {
        self.symbols.iter()
            .filter(|(name, _)| name.contains(substring))
            .map(|(_, sym)| sym)
            .collect()
    }

    pub fn add_symbol(&mut self, name: String, addr: Address, size: Option<u64>, symbol_type: SymbolType) {
        let symbol = Symbol {
            name: name.clone(),
            address: addr,
            size,
            symbol_type,
            demangled_name: None,
        };

        self.address_to_symbol.insert(addr.as_u64(), name.clone());
        self.symbols.insert(name, symbol);
    }

    pub fn get_symbols(&self) -> Result<Vec<Symbol>, MemoryError> {
        Ok(self.symbols.values().cloned().collect())
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    pub fn get_nearest_symbol(&self, addr: Address) -> Option<(&Symbol, i64)> {
        let addr_u64 = addr.as_u64();
        let mut nearest: Option<(&Symbol, i64)> = None;

        for symbol in self.symbols.values() {
            let diff = addr_u64 as i64 - symbol.address.as_u64() as i64;

            if diff >= 0 {
                if let Some((_, current_diff)) = nearest {
                    if diff < current_diff {
                        nearest = Some((symbol, diff));
                    }
                } else {
                    nearest = Some((symbol, diff));
                }
            }
        }

        nearest
    }

    pub fn format_address(&self, addr: Address) -> String {
        if let Some((symbol, offset)) = self.get_nearest_symbol(addr) {
            if offset == 0 {
                symbol.name.clone()
            } else {
                format!("{}+0x{:x}", symbol.name, offset)
            }
        } else {
            format!("0x{:016x}", addr.as_u64())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: Address,
    pub size: Option<u64>,
    pub symbol_type: SymbolType,
    pub demangled_name: Option<String>,
}

impl Symbol {
    pub fn new(name: String, address: Address) -> Self {
        Self {
            name,
            address,
            size: None,
            symbol_type: SymbolType::Unknown,
            demangled_name: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_type(mut self, symbol_type: SymbolType) -> Self {
        self.symbol_type = symbol_type;
        self
    }

    pub fn display_name(&self) -> &str {
        self.demangled_name.as_deref().unwrap_or(&self.name)
    }

    pub fn is_function(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Function)
    }

    pub fn is_data(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Data | SymbolType::BSS)
    }

    pub fn contains(&self, addr: Address) -> bool {
        if let Some(size) = self.size {
            let start = self.address.as_u64();
            let end = start + size;
            let addr = addr.as_u64();
            addr >= start && addr < end
        } else {
            self.address == addr
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Data,
    BSS,
    External,
    Undefined,
    Section,
    Unknown,
}

impl SymbolType {
    pub fn from_nlist_type(n_type: u8) -> Self {
        match n_type & 0x0E {
            0x00 => SymbolType::Undefined,
            0x02 => SymbolType::External,
            0x04 => SymbolType::Data,
            0x06 => SymbolType::BSS,
            0x0E => SymbolType::Section,
            _ => SymbolType::Unknown,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SymbolType::Function => "function",
            SymbolType::Data => "data",
            SymbolType::BSS => "bss",
            SymbolType::External => "external",
            SymbolType::Undefined => "undefined",
            SymbolType::Section => "section",
            SymbolType::Unknown => "unknown",
        }
    }
}

pub fn demangle_symbol(mangled: &str) -> Option<String> {
    if mangled.starts_with("_Z") {
        demangle_itanium(mangled)
    } else if mangled.starts_with("__Z") {
        demangle_itanium(&mangled[1..])
    } else {
        None
    }
}

fn demangle_itanium(mangled: &str) -> Option<String> {
    let mangled = if mangled.starts_with("_Z") {
        &mangled[2..]
    } else {
        return None;
    };

    let mut result = String::new();
    let mut chars = mangled.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut len_str = String::new();
            len_str.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    len_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if let Ok(len) = len_str.parse::<usize>() {
                let name: String = chars.by_ref().take(len).collect();
                if !result.is_empty() {
                    result.push_str("::");
                }
                result.push_str(&name);
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub struct SymbolCache {
    resolver: SymbolResolver,
    cache: HashMap<u64, Option<Symbol>>,
}

impl SymbolCache {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            resolver: SymbolResolver::new(reader),
            cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, addr: Address) -> Option<&Symbol> {
        let addr_u64 = addr.as_u64();

        if !self.cache.contains_key(&addr_u64) {
            let symbol = self.resolver.resolve_address(addr).cloned();
            self.cache.insert(addr_u64, symbol);
        }

        self.cache.get(&addr_u64).and_then(|s| s.as_ref())
    }

    pub fn preload(&mut self) -> Result<usize, MemoryError> {
        self.resolver.load_symbols()
    }
}
