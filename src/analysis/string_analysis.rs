// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::collections::HashMap;
use std::sync::Arc;

pub struct StringAnalyzer {
    reader: Arc<dyn MemoryReader>,
    strings: HashMap<u64, FoundString>,
    min_length: usize,
    max_length: usize,
}

#[derive(Debug, Clone)]
pub struct FoundString {
    pub address: Address,
    pub value: String,
    pub encoding: StringEncoding,
    pub length: usize,
    pub xrefs: Vec<Address>,
    pub category: StringCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Ascii,
    Utf8,
    Utf16Le,
    Utf16Be,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringCategory {
    FunctionName,
    ClassName,
    PropertyName,
    ErrorMessage,
    FilePath,
    Url,
    Generic,
    Unknown,
}

impl StringAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            strings: HashMap::new(),
            min_length: 4,
            max_length: 4096,
        }
    }

    pub fn with_min_length(mut self, len: usize) -> Self {
        self.min_length = len;
        self
    }

    pub fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    pub fn scan(&mut self, start: Address, end: Address) -> Result<Vec<FoundString>, MemoryError> {
        let mut found = Vec::new();
        let scan_size = (end.as_u64() - start.as_u64()) as usize;
        let data = self.reader.read_bytes(start, scan_size)?;

        let mut i = 0;
        while i < data.len() {
            if let Some(string_info) = self.try_extract_string(&data, i) {
                let addr = start + i as u64;
                let category = self.categorize_string(&string_info.0);

                let fs = FoundString {
                    address: addr,
                    value: string_info.0,
                    encoding: string_info.1,
                    length: string_info.2,
                    xrefs: Vec::new(),
                    category,
                };

                self.strings.insert(addr.as_u64(), fs.clone());
                found.push(fs);

                i += string_info.2;
            } else {
                i += 1;
            }
        }

        Ok(found)
    }

    fn try_extract_string(&self, data: &[u8], offset: usize) -> Option<(String, StringEncoding, usize)> {
        if let Some(result) = self.try_extract_ascii(data, offset) {
            return Some(result);
        }

        if let Some(result) = self.try_extract_utf16le(data, offset) {
            return Some(result);
        }

        None
    }

    fn try_extract_ascii(&self, data: &[u8], offset: usize) -> Option<(String, StringEncoding, usize)> {
        let mut len = 0;
        let max = (data.len() - offset).min(self.max_length);

        while len < max {
            let byte = data[offset + len];
            if byte == 0 {
                break;
            }
            if !is_printable_ascii(byte) {
                return None;
            }
            len += 1;
        }

        if len < self.min_length {
            return None;
        }

        if offset + len >= data.len() || data[offset + len] != 0 {
            return None;
        }

        let s = String::from_utf8(data[offset..offset + len].to_vec()).ok()?;
        Some((s, StringEncoding::Ascii, len + 1))
    }

    fn try_extract_utf16le(&self, data: &[u8], offset: usize) -> Option<(String, StringEncoding, usize)> {
        if (data.len() - offset) < 4 {
            return None;
        }

        let mut chars = Vec::new();
        let max_chars = (data.len() - offset) / 2;
        let max_chars = max_chars.min(self.max_length);

        for i in 0..max_chars {
            let byte_offset = offset + i * 2;
            if byte_offset + 1 >= data.len() {
                break;
            }

            let code_unit = u16::from_le_bytes([data[byte_offset], data[byte_offset + 1]]);

            if code_unit == 0 {
                break;
            }

            if code_unit < 0x20 || (code_unit >= 0x7F && code_unit < 0xA0) {
                return None;
            }

            chars.push(code_unit);
        }

        if chars.len() < self.min_length {
            return None;
        }

        let s = String::from_utf16(&chars).ok()?;
        let byte_len = (chars.len() + 1) * 2;

        Some((s, StringEncoding::Utf16Le, byte_len))
    }

    fn categorize_string(&self, s: &str) -> StringCategory {
        let s_lower = s.to_lowercase();

        if s.starts_with("__") || s.contains("::") {
            return StringCategory::FunctionName;
        }

        if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) &&
           !s.contains(' ') && s.len() < 64 {
            if s.contains("Error") || s.contains("Exception") {
                return StringCategory::ErrorMessage;
            }
            return StringCategory::ClassName;
        }

        if s_lower.contains("error") || s_lower.contains("fail") ||
           s_lower.contains("invalid") || s_lower.contains("cannot") {
            return StringCategory::ErrorMessage;
        }

        if s.contains('/') || s.contains('\\') || s.ends_with(".lua") ||
           s.ends_with(".rbxl") || s.ends_with(".rbxm") {
            return StringCategory::FilePath;
        }

        if s.starts_with("http://") || s.starts_with("https://") ||
           s.starts_with("rbxasset://") || s.starts_with("rbxassetid://") {
            return StringCategory::Url;
        }

        if s.chars().all(|c| c.is_alphanumeric() || c == '_') &&
           s.len() < 64 && !s.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            return StringCategory::PropertyName;
        }

        StringCategory::Generic
    }

    pub fn get_string(&self, addr: Address) -> Option<&FoundString> {
        self.strings.get(&addr.as_u64())
    }

    pub fn get_all_strings(&self) -> Vec<&FoundString> {
        self.strings.values().collect()
    }

    pub fn get_strings_by_category(&self, category: StringCategory) -> Vec<&FoundString> {
        self.strings
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    pub fn find_string(&self, target: &str) -> Vec<&FoundString> {
        self.strings
            .values()
            .filter(|s| s.value.contains(target))
            .collect()
    }

    pub fn find_string_exact(&self, target: &str) -> Option<&FoundString> {
        self.strings.values().find(|s| s.value == target)
    }

    pub fn get_function_names(&self) -> Vec<&FoundString> {
        self.get_strings_by_category(StringCategory::FunctionName)
    }

    pub fn get_class_names(&self) -> Vec<&FoundString> {
        self.get_strings_by_category(StringCategory::ClassName)
    }

    pub fn get_error_messages(&self) -> Vec<&FoundString> {
        self.get_strings_by_category(StringCategory::ErrorMessage)
    }

    pub fn add_xref(&mut self, string_addr: Address, xref: Address) {
        if let Some(fs) = self.strings.get_mut(&string_addr.as_u64()) {
            if !fs.xrefs.contains(&xref) {
                fs.xrefs.push(xref);
            }
        }
    }

    pub fn clear(&mut self) {
        self.strings.clear();
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn export_strings(&self) -> Vec<(u64, String)> {
        let mut strings: Vec<_> = self.strings
            .iter()
            .map(|(&addr, fs)| (addr, fs.value.clone()))
            .collect();
        strings.sort_by_key(|(addr, _)| *addr);
        strings
    }
}

fn is_printable_ascii(byte: u8) -> bool {
    byte >= 0x20 && byte < 0x7F || byte == 0x09 || byte == 0x0A || byte == 0x0D
}

pub fn find_string_in_range(reader: &dyn MemoryReader, target: &str, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
    let mut results = Vec::new();
    let target_bytes = target.as_bytes();
    let scan_size = (end.as_u64() - start.as_u64()) as usize;
    let data = reader.read_bytes(start, scan_size)?;

    for i in 0..data.len().saturating_sub(target_bytes.len()) {
        if &data[i..i + target_bytes.len()] == target_bytes {
            results.push(start + i as u64);
        }
    }

    Ok(results)
}

pub fn read_c_string(reader: &dyn MemoryReader, addr: Address, max_len: usize) -> Result<String, MemoryError> {
    let bytes = reader.read_bytes(addr, max_len)?;
    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..null_pos].to_vec())
        .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
}

pub fn read_pascal_string(reader: &dyn MemoryReader, addr: Address) -> Result<String, MemoryError> {
    let len = reader.read_u8(addr)? as usize;
    let bytes = reader.read_bytes(addr + 1, len)?;
    String::from_utf8(bytes)
        .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
}

pub fn read_length_prefixed_string(reader: &dyn MemoryReader, addr: Address) -> Result<String, MemoryError> {
    let len = reader.read_u32(addr)? as usize;
    if len > 65536 {
        return Err(MemoryError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "String too long",
        )));
    }
    let bytes = reader.read_bytes(addr + 4, len)?;
    String::from_utf8(bytes)
        .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
}

pub fn is_valid_string_pointer(reader: &dyn MemoryReader, addr: Address) -> bool {
    match reader.read_u64(addr) {
        Ok(ptr) => {
            if ptr < 0x100000000 || ptr >= 0x800000000000 {
                return false;
            }
            let ptr_addr = Address::new(ptr);
            if let Ok(bytes) = reader.read_bytes(ptr_addr, 8) {
                bytes.iter().take_while(|&&b| is_printable_ascii(b)).count() >= 4
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
