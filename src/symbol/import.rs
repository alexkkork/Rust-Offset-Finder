// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    pub name: String,
    pub library: String,
    pub address: Address,
    pub ordinal: Option<u32>,
    pub is_lazy: bool,
}

pub struct ImportTable {
    imports: HashMap<String, Vec<ImportedSymbol>>,
    by_name: HashMap<String, ImportedSymbol>,
    by_address: HashMap<u64, String>,
}

impl ImportTable {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
            by_name: HashMap::new(),
            by_address: HashMap::new(),
        }
    }

    pub fn add(&mut self, import: ImportedSymbol) {
        self.by_name.insert(import.name.clone(), import.clone());
        self.by_address.insert(import.address.as_u64(), import.name.clone());

        self.imports.entry(import.library.clone())
            .or_default()
            .push(import);
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ImportedSymbol> {
        self.by_name.get(name)
    }

    pub fn get_by_address(&self, addr: Address) -> Option<&ImportedSymbol> {
        self.by_address.get(&addr.as_u64())
            .and_then(|name| self.by_name.get(name))
    }

    pub fn get_from_library(&self, library: &str) -> Option<&Vec<ImportedSymbol>> {
        self.imports.get(library)
    }

    pub fn libraries(&self) -> impl Iterator<Item = &String> {
        self.imports.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ImportedSymbol> {
        self.by_name.values()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    pub fn library_count(&self) -> usize {
        self.imports.len()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    pub fn find_prefix(&self, prefix: &str) -> Vec<&ImportedSymbol> {
        self.by_name.iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(_, import)| import)
            .collect()
    }

    pub fn find_suffix(&self, suffix: &str) -> Vec<&ImportedSymbol> {
        self.by_name.iter()
            .filter(|(name, _)| name.ends_with(suffix))
            .map(|(_, import)| import)
            .collect()
    }

    pub fn find_contains(&self, substring: &str) -> Vec<&ImportedSymbol> {
        self.by_name.iter()
            .filter(|(name, _)| name.contains(substring))
            .map(|(_, import)| import)
            .collect()
    }

    pub fn find_from_library_prefix(&self, library: &str, prefix: &str) -> Vec<&ImportedSymbol> {
        self.imports.get(library)
            .map(|imports| {
                imports.iter()
                    .filter(|i| i.name.starts_with(prefix))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for ImportTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportedSymbol {
    pub fn new(name: String, library: String, address: Address) -> Self {
        Self {
            name,
            library,
            address,
            ordinal: None,
            is_lazy: false,
        }
    }

    pub fn with_ordinal(mut self, ordinal: u32) -> Self {
        self.ordinal = Some(ordinal);
        self
    }

    pub fn set_lazy(mut self, lazy: bool) -> Self {
        self.is_lazy = lazy;
        self
    }
}

pub fn common_system_libraries() -> Vec<&'static str> {
    vec![
        "libSystem.B.dylib",
        "libc++.1.dylib",
        "libobjc.A.dylib",
        "Foundation",
        "CoreFoundation",
        "AppKit",
        "UIKit",
        "Security",
        "CoreGraphics",
        "QuartzCore",
        "Metal",
        "IOKit",
    ]
}

pub fn is_system_library(name: &str) -> bool {
    name.starts_with("/usr/lib/")
        || name.starts_with("/System/Library/")
        || name.contains("dylib")
        || common_system_libraries().iter().any(|lib| name.contains(lib))
}
