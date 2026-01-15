// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use super::types::{FFlag, FFlagType, FFlagValue, FFlagCollection, FFlagCategory};
use std::sync::Arc;
use std::collections::HashMap;

pub struct FFlagParser {
    reader: Arc<dyn MemoryReader>,
}

impl FFlagParser {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn parse_flag_table(&self, table_addr: Address, count: usize) -> Result<Vec<FFlag>, MemoryError> {
        let mut flags = Vec::new();
        let entry_size = 0x18;

        for i in 0..count {
            let entry_addr = table_addr + (i as u64 * entry_size);

            if let Ok(Some(flag)) = self.parse_flag_entry(entry_addr) {
                flags.push(flag);
            }
        }

        Ok(flags)
    }

    pub fn parse_flag_entry(&self, addr: Address) -> Result<Option<FFlag>, MemoryError> {
        let name_ptr = self.reader.read_u64(addr)?;
        let value_ptr = self.reader.read_u64(addr + 8)?;
        let type_info = self.reader.read_u64(addr + 16)?;

        if name_ptr == 0 || name_ptr < 0x1000 {
            return Ok(None);
        }

        let full_name = self.read_cstring(Address::new(name_ptr))?;

        let (flag_type, name) = self.parse_flag_name(&full_name)?;

        let value = self.read_flag_value(Address::new(value_ptr), &flag_type)?;

        Ok(Some(FFlag::new(name, flag_type, value, addr.as_u64())))
    }

    fn parse_flag_name(&self, full_name: &str) -> Result<(FFlagType, String), MemoryError> {
        let prefixes = [
            ("DFFlag", FFlagType::DFFlag),
            ("DFInt", FFlagType::DFInt),
            ("DFString", FFlagType::DFString),
            ("DFLog", FFlagType::DFLog),
            ("SFFlag", FFlagType::SFFlag),
            ("SFInt", FFlagType::SFInt),
            ("SFString", FFlagType::SFString),
            ("SFLog", FFlagType::SFLog),
            ("FFlag", FFlagType::FFlag),
            ("FInt", FFlagType::FInt),
            ("FString", FFlagType::FString),
            ("FLog", FFlagType::FLog),
        ];

        for (prefix, flag_type) in &prefixes {
            if full_name.starts_with(prefix) {
                let name = full_name[prefix.len()..].to_string();
                return Ok((*flag_type, name));
            }
        }

        Ok((FFlagType::Unknown, full_name.to_string()))
    }

    fn read_flag_value(&self, addr: Address, flag_type: &FFlagType) -> Result<FFlagValue, MemoryError> {
        if addr.as_u64() == 0 {
            return Ok(FFlagValue::Unknown);
        }

        match flag_type {
            FFlagType::FFlag | FFlagType::DFFlag | FFlagType::SFFlag => {
                let val = self.reader.read_u8(addr)?;
                Ok(FFlagValue::Bool(val != 0))
            }
            FFlagType::FInt | FFlagType::DFInt | FFlagType::SFInt => {
                let val = self.reader.read_i64(addr)?;
                Ok(FFlagValue::Int(val))
            }
            FFlagType::FString | FFlagType::DFString | FFlagType::SFString => {
                let str_ptr = self.reader.read_u64(addr)?;
                if str_ptr != 0 {
                    let s = self.read_cstring(Address::new(str_ptr))?;
                    Ok(FFlagValue::String(s))
                } else {
                    Ok(FFlagValue::String(String::new()))
                }
            }
            FFlagType::FLog | FFlagType::DFLog | FFlagType::SFLog => {
                let val = self.reader.read_i32(addr)?;
                Ok(FFlagValue::Log(val))
            }
            FFlagType::Unknown => Ok(FFlagValue::Unknown),
        }
    }

    fn read_cstring(&self, addr: Address) -> Result<String, MemoryError> {
        let mut bytes = Vec::new();
        let mut current = addr;

        for _ in 0..1024 {
            let byte = self.reader.read_u8(current)?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            current = current + 1;
        }

        String::from_utf8(bytes).map_err(|_| MemoryError::InvalidString)
    }

    pub fn categorize_flags(&self, flags: &[FFlag]) -> Vec<FFlagCategory> {
        let mut categories: HashMap<String, FFlagCategory> = HashMap::new();

        for flag in flags {
            let category_name = self.extract_category(&flag.name);

            categories
                .entry(category_name.clone())
                .or_insert_with(|| FFlagCategory::new(&category_name))
                .add(flag.clone());
        }

        let mut result: Vec<FFlagCategory> = categories.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    fn extract_category(&self, name: &str) -> String {
        let parts: Vec<&str> = name.split(|c: char| c.is_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            let first_upper: String = name.chars()
                .take_while(|c| c.is_uppercase() || c.is_numeric())
                .collect();

            if first_upper.is_empty() {
                "Misc".to_string()
            } else {
                first_upper
            }
        } else {
            parts[0].to_string()
        }
    }

    pub fn find_registration_function(&self, collection: &FFlagCollection) -> Option<Address> {
        if collection.flags.is_empty() {
            return None;
        }

        let first_flag = &collection.flags[0];
        let flag_addr = Address::new(first_flag.address);

        if let Ok(regions) = self.reader.get_regions() {
            for region in &regions {
                if !region.is_executable() {
                    continue;
                }

                let start = region.start().as_u64();
                let end = region.end().as_u64();

                if flag_addr.as_u64() >= start && flag_addr.as_u64() < end {
                    return Some(region.start());
                }
            }
        }

        None
    }
}
