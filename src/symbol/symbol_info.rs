// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    name: String,
    address: Address,
    size: u64,
    kind: SymbolKind,
    demangled: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Data,
    Undefined,
    Absolute,
}

impl SymbolInfo {
    pub fn new(name: String, address: Address, kind: SymbolKind) -> Self {
        Self {
            name,
            address,
            size: 0,
            kind,
            demangled: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn with_demangled(mut self, demangled: String) -> Self {
        self.demangled = Some(demangled);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    pub fn demangled(&self) -> Option<&str> {
        self.demangled.as_deref()
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function)
    }

    pub fn is_data(&self) -> bool {
        matches!(self.kind, SymbolKind::Data)
    }
}

impl fmt::Display for SymbolInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(demangled) = &self.demangled {
            write!(f, "{} ({}) @ {}", demangled, self.name, self.address)
        } else {
            write!(f, "{} @ {}", self.name, self.address)
        }
    }
}
