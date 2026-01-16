// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use super::types::{FFlag, FFlagType, FFlagValue, FFlagCollection};
use std::sync::Arc;
use std::collections::HashSet;

pub struct FFlagFinder {
    reader: Arc<dyn MemoryReader>,
    found_names: HashSet<String>,
}

impl FFlagFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            found_names: HashSet::new(),
        }
    }

    pub fn find_all(&mut self, start: Address, end: Address) -> Result<FFlagCollection, MemoryError> {
        let mut collection = FFlagCollection::new();

        // Find flags by scanning for string patterns
        self.find_by_string_scan(start, end, &mut collection)?;

        Ok(collection)
    }

    fn find_by_string_scan(&mut self, start: Address, end: Address, collection: &mut FFlagCollection) -> Result<(), MemoryError> {
        let prefixes = [
            "FFlag", "FInt", "FString", "FLog",
            "DFFlag", "DFInt", "DFString", "DFLog",
            "SFFlag", "SFInt", "SFString", "SFLog",
        ];

        let size = (end.as_u64() - start.as_u64()).min(100_000_000) as usize;
        let data = self.reader.read_bytes(start, size)?;

        // Scan for each prefix
        for prefix in &prefixes {
            let prefix_bytes = prefix.as_bytes();
            
            for (i, window) in data.windows(prefix_bytes.len()).enumerate() {
                if window == prefix_bytes {
                    // Found a potential flag
                    let flag_start = i;
                    
                    // Read the full flag name (up to 128 bytes or null terminator)
                    let mut name_end = flag_start + prefix_bytes.len();
                    while name_end < data.len() && name_end < flag_start + 128 {
                        let b = data[name_end];
                        if b == 0 || !is_valid_flag_char(b) {
                            break;
                        }
                        name_end += 1;
                    }

                    if name_end > flag_start + prefix_bytes.len() {
                        let flag_name = String::from_utf8_lossy(&data[flag_start..name_end]).to_string();
                        
                        if self.is_valid_flag_name(&flag_name) && !self.found_names.contains(&flag_name) {
                            self.found_names.insert(flag_name.clone());

                            let flag_type = FFlagType::from_prefix(prefix);
                            let name_part = flag_name[prefix.len()..].to_string();
                            let addr = start.as_u64() + flag_start as u64;

                            let flag = FFlag::new(
                                name_part,
                                flag_type,
                                FFlagValue::Unknown,
                                addr,
                            );

                            collection.add(flag);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn is_valid_flag_name(&self, name: &str) -> bool {
        if name.len() < 5 || name.len() > 128 {
            return false;
        }

        // Must start with a known prefix
        let valid_prefixes = ["FFlag", "FInt", "FString", "FLog", "DFFlag", "DFInt", "DFString", "DFLog", "SFFlag", "SFInt", "SFString", "SFLog"];
        
        if !valid_prefixes.iter().any(|p| name.starts_with(p)) {
            return false;
        }

        // After prefix, should have alphanumeric characters
        true
    }

    pub fn search_for_flag(&self, flag_name: &str, start: Address, end: Address) -> Result<Option<Address>, MemoryError> {
        let size = (end.as_u64() - start.as_u64()).min(100_000_000) as usize;
        let data = self.reader.read_bytes(start, size)?;
        let search_bytes = flag_name.as_bytes();

        for (i, window) in data.windows(search_bytes.len()).enumerate() {
            if window == search_bytes {
                return Ok(Some(Address::new(start.as_u64() + i as u64)));
            }
        }

        Ok(None)
    }

    pub fn get_found_count(&self) -> usize {
        self.found_names.len()
    }
}

fn is_valid_flag_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_flag_names() {
        // Test that flag names are validated correctly
        assert!(is_valid_flag_char(b'A'));
        assert!(is_valid_flag_char(b'z'));
        assert!(is_valid_flag_char(b'0'));
        assert!(is_valid_flag_char(b'_'));
        assert!(!is_valid_flag_char(b' '));
        assert!(!is_valid_flag_char(b'.'));
    }
}
