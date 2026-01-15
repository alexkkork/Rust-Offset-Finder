// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::pattern::PatternMatcher;
use crate::analysis::StringAnalyzer;
use super::types::{FFlag, FFlagType, FFlagValue, FFlagCollection};
use std::sync::Arc;
use std::collections::HashSet;

pub struct FFlagFinder {
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: Arc<PatternMatcher>,
    string_analyzer: StringAnalyzer,
    found_names: HashSet<String>,
}

impl FFlagFinder {
    pub fn new(reader: Arc<dyn MemoryReader>, pattern_matcher: Arc<PatternMatcher>) -> Self {
        Self {
            reader: reader.clone(),
            pattern_matcher,
            string_analyzer: StringAnalyzer::new(reader),
            found_names: HashSet::new(),
        }
    }

    pub fn find_all(&mut self) -> Result<FFlagCollection, MemoryError> {
        let mut collection = FFlagCollection::new();

        self.find_by_string_patterns(&mut collection)?;
        self.find_by_vtable_scan(&mut collection)?;
        self.find_by_registration_pattern(&mut collection)?;

        Ok(collection)
    }

    fn find_by_string_patterns(&mut self, collection: &mut FFlagCollection) -> Result<(), MemoryError> {
        let prefixes = [
            "FFlag", "FInt", "FString", "FLog",
            "DFFlag", "DFInt", "DFString", "DFLog",
            "SFFlag", "SFInt", "SFString", "SFLog",
        ];

        let strings = self.string_analyzer.find_all_strings(4, 256)?;

        for string_info in &strings {
            let s = &string_info.value;

            for prefix in &prefixes {
                if s.starts_with(prefix) && s.len() > prefix.len() {
                    let name = s[prefix.len()..].to_string();

                    if self.is_valid_flag_name(&name) && !self.found_names.contains(s) {
                        self.found_names.insert(s.clone());

                        let flag_type = FFlagType::from_prefix(prefix);
                        let value = self.try_read_flag_value(string_info.address, &flag_type)?;

                        let flag = FFlag::new(
                            name,
                            flag_type,
                            value,
                            string_info.address.as_u64(),
                        );

                        collection.add(flag);
                    }
                }
            }
        }

        Ok(())
    }

    fn find_by_vtable_scan(&mut self, collection: &mut FFlagCollection) -> Result<(), MemoryError> {
        let flag_vtable_pattern = "?? ?? ?? ?? ?? ?? 00 00 01 00 00 00 00 00 00 00";

        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.is_readable() {
                continue;
            }

            let matches = self.pattern_matcher.find_pattern(
                &flag_vtable_pattern.parse().unwrap_or_default(),
                region.start(),
                region.size() as usize,
            )?;

            for addr in matches {
                if let Ok(Some(flag)) = self.try_parse_flag_at(addr) {
                    if !self.found_names.contains(&flag.name) {
                        self.found_names.insert(format!("{}{}", flag.prefix(), flag.name));
                        collection.add(flag);
                    }
                }
            }
        }

        Ok(())
    }

    fn find_by_registration_pattern(&mut self, collection: &mut FFlagCollection) -> Result<(), MemoryError> {
        let registration_patterns = [
            "?? ?? ?? ?? 00 00 00 00 ?? ?? ?? ?? ?? ?? ?? ?? 00 00 00 00",
            "48 8D ?? ?? ?? ?? ?? 48 89 ?? ?? 48 8D ?? ?? ?? ?? ??",
        ];

        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.is_executable() {
                continue;
            }

            for pattern_str in &registration_patterns {
                let pattern = match pattern_str.parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let matches = self.pattern_matcher.find_pattern(
                    &pattern,
                    region.start(),
                    region.size() as usize,
                )?;

                for addr in matches {
                    if let Ok(Some(flag)) = self.try_parse_registration_at(addr) {
                        if !self.found_names.contains(&flag.name) {
                            self.found_names.insert(format!("{}{}", flag.prefix(), flag.name));
                            collection.add(flag);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn is_valid_flag_name(&self, name: &str) -> bool {
        if name.is_empty() || name.len() > 200 {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_uppercase() && !first_char.is_ascii_lowercase() {
            return false;
        }

        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn try_read_flag_value(&self, addr: Address, flag_type: &FFlagType) -> Result<FFlagValue, MemoryError> {
        if let Ok(ptr) = self.reader.read_u64(addr + 8) {
            if ptr != 0 {
                match flag_type {
                    FFlagType::FFlag | FFlagType::DFFlag | FFlagType::SFFlag => {
                        let val = self.reader.read_u8(Address::new(ptr))?;
                        return Ok(FFlagValue::Bool(val != 0));
                    }
                    FFlagType::FInt | FFlagType::DFInt | FFlagType::SFInt => {
                        let val = self.reader.read_i64(Address::new(ptr))?;
                        return Ok(FFlagValue::Int(val));
                    }
                    FFlagType::FString | FFlagType::DFString | FFlagType::SFString => {
                        if let Ok(s) = self.read_string_at(Address::new(ptr)) {
                            return Ok(FFlagValue::String(s));
                        }
                    }
                    FFlagType::FLog | FFlagType::DFLog | FFlagType::SFLog => {
                        let val = self.reader.read_i32(Address::new(ptr))?;
                        return Ok(FFlagValue::Log(val));
                    }
                    _ => {}
                }
            }
        }

        Ok(FFlagValue::Unknown)
    }

    fn try_parse_flag_at(&self, addr: Address) -> Result<Option<FFlag>, MemoryError> {
        let name_ptr = self.reader.read_u64(addr)?;

        if name_ptr == 0 || name_ptr < 0x1000 {
            return Ok(None);
        }

        let name = match self.read_string_at(Address::new(name_ptr)) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        let prefixes = [
            "FFlag", "FInt", "FString", "FLog",
            "DFFlag", "DFInt", "DFString", "DFLog",
            "SFFlag", "SFInt", "SFString", "SFLog",
        ];

        for prefix in &prefixes {
            if name.starts_with(prefix) {
                let flag_name = name[prefix.len()..].to_string();
                let flag_type = FFlagType::from_prefix(prefix);
                let value = self.try_read_flag_value(addr, &flag_type)?;

                return Ok(Some(FFlag::new(flag_name, flag_type, value, addr.as_u64())));
            }
        }

        Ok(None)
    }

    fn try_parse_registration_at(&self, addr: Address) -> Result<Option<FFlag>, MemoryError> {
        self.try_parse_flag_at(addr)
    }

    fn read_string_at(&self, addr: Address) -> Result<String, MemoryError> {
        let mut bytes = Vec::new();
        let mut current = addr;

        for _ in 0..512 {
            let byte = self.reader.read_u8(current)?;
            if byte == 0 {
                break;
            }
            if byte < 0x20 || byte > 0x7E {
                return Err(MemoryError::InvalidString);
            }
            bytes.push(byte);
            current = current + 1;
        }

        String::from_utf8(bytes).map_err(|_| MemoryError::InvalidString)
    }

    pub fn find_flag_by_name(&self, name: &str) -> Result<Option<FFlag>, MemoryError> {
        let full_names = [
            format!("FFlag{}", name),
            format!("FInt{}", name),
            format!("FString{}", name),
            format!("DFFlag{}", name),
            format!("DFInt{}", name),
            format!("DFString{}", name),
            format!("SFFlag{}", name),
            format!("SFInt{}", name),
            format!("SFString{}", name),
        ];

        let strings = self.string_analyzer.find_all_strings(4, 256)?;

        for string_info in &strings {
            for full_name in &full_names {
                if &string_info.value == full_name {
                    let prefix = if full_name.starts_with("FFlag") { "FFlag" }
                        else if full_name.starts_with("FInt") { "FInt" }
                        else if full_name.starts_with("FString") { "FString" }
                        else if full_name.starts_with("DFFlag") { "DFFlag" }
                        else if full_name.starts_with("DFInt") { "DFInt" }
                        else if full_name.starts_with("DFString") { "DFString" }
                        else if full_name.starts_with("SFFlag") { "SFFlag" }
                        else if full_name.starts_with("SFInt") { "SFInt" }
                        else { "SFString" };

                    let flag_type = FFlagType::from_prefix(prefix);
                    let value = self.try_read_flag_value(string_info.address, &flag_type)?;

                    return Ok(Some(FFlag::new(
                        name.to_string(),
                        flag_type,
                        value,
                        string_info.address.as_u64(),
                    )));
                }
            }
        }

        Ok(None)
    }
}
