// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError, MemoryRegion};
use std::sync::Arc;
use std::collections::HashMap;

pub struct StringAnalyzer {
    reader: Arc<dyn MemoryReader>,
    min_length: usize,
    max_length: usize,
}

impl StringAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            min_length: 4,
            max_length: 4096,
        }
    }

    pub fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    pub fn find_strings_in_region(&self, region: &MemoryRegion) -> Result<Vec<FoundString>, MemoryError> {
        let mut strings = Vec::new();

        let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

        let mut current_start = 0;
        let mut current_string = Vec::new();

        for (offset, &byte) in data.iter().enumerate() {
            if self.is_printable_ascii(byte) {
                if current_string.is_empty() {
                    current_start = offset;
                }
                current_string.push(byte);
            } else {
                if current_string.len() >= self.min_length && current_string.len() <= self.max_length {
                    let addr = Address::new(region.range.start.as_u64() + current_start as u64);
                    strings.push(FoundString {
                        address: addr,
                        content: String::from_utf8_lossy(&current_string).to_string(),
                        encoding: StringEncoding::Ascii,
                        is_null_terminated: byte == 0,
                    });
                }
                current_string.clear();
            }
        }

        if current_string.len() >= self.min_length && current_string.len() <= self.max_length {
            let addr = Address::new(region.range.start.as_u64() + current_start as u64);
            strings.push(FoundString {
                address: addr,
                content: String::from_utf8_lossy(&current_string).to_string(),
                encoding: StringEncoding::Ascii,
                is_null_terminated: false,
            });
        }

        Ok(strings)
    }

    pub fn find_utf8_strings_in_region(&self, region: &MemoryRegion) -> Result<Vec<FoundString>, MemoryError> {
        let mut strings = Vec::new();

        let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

        let mut current_start = 0;
        let mut current_bytes = Vec::new();

        for (offset, &byte) in data.iter().enumerate() {
            if byte == 0 || self.is_valid_utf8_byte(byte, &current_bytes) {
                if byte == 0 {
                    if let Ok(s) = String::from_utf8(current_bytes.clone()) {
                        if s.len() >= self.min_length && s.len() <= self.max_length {
                            let addr = Address::new(region.range.start.as_u64() + current_start as u64);
                            strings.push(FoundString {
                                address: addr,
                                content: s,
                                encoding: StringEncoding::Utf8,
                                is_null_terminated: true,
                            });
                        }
                    }
                    current_bytes.clear();
                    current_start = offset + 1;
                } else {
                    if current_bytes.is_empty() {
                        current_start = offset;
                    }
                    current_bytes.push(byte);
                }
            } else {
                current_bytes.clear();
                current_start = offset + 1;
            }
        }

        Ok(strings)
    }

    pub fn find_wide_strings_in_region(&self, region: &MemoryRegion) -> Result<Vec<FoundString>, MemoryError> {
        let mut strings = Vec::new();

        let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

        if data.len() < 2 {
            return Ok(strings);
        }

        let mut current_start = 0;
        let mut current_chars = Vec::new();

        let mut i = 0;
        while i + 1 < data.len() {
            let wide_char = u16::from_le_bytes([data[i], data[i + 1]]);

            if wide_char == 0 {
                if current_chars.len() >= self.min_length && current_chars.len() <= self.max_length {
                    let addr = Address::new(region.range.start.as_u64() + current_start as u64);
                    strings.push(FoundString {
                        address: addr,
                        content: String::from_utf16_lossy(&current_chars),
                        encoding: StringEncoding::Utf16Le,
                        is_null_terminated: true,
                    });
                }
                current_chars.clear();
                current_start = i + 2;
            } else if self.is_printable_wide(wide_char) {
                if current_chars.is_empty() {
                    current_start = i;
                }
                current_chars.push(wide_char);
            } else {
                current_chars.clear();
                current_start = i + 2;
            }

            i += 2;
        }

        Ok(strings)
    }

    pub fn read_string_at(&self, addr: Address) -> Result<Option<String>, MemoryError> {
        let mut result = Vec::new();

        for offset in 0..self.max_length {
            let byte = self.reader.read_u8(addr + offset as u64)?;
            if byte == 0 {
                break;
            }
            if !self.is_printable_ascii(byte) && byte < 0x80 {
                return Ok(None);
            }
            result.push(byte);
        }

        if result.len() < self.min_length {
            return Ok(None);
        }

        String::from_utf8(result).ok().map(Some).unwrap_or(Ok(None))
    }

    pub fn read_wide_string_at(&self, addr: Address) -> Result<Option<String>, MemoryError> {
        let mut chars = Vec::new();

        for offset in (0..self.max_length * 2).step_by(2) {
            let wide_char = self.reader.read_u16(addr + offset as u64)?;
            if wide_char == 0 {
                break;
            }
            if !self.is_printable_wide(wide_char) {
                return Ok(None);
            }
            chars.push(wide_char);
        }

        if chars.len() < self.min_length {
            return Ok(None);
        }

        Ok(Some(String::from_utf16_lossy(&chars)))
    }

    pub fn search_for_string(&self, target: &str, regions: &[MemoryRegion]) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();
        let target_bytes = target.as_bytes();

        for region in regions {
            if !region.protection.is_readable() {
                continue;
            }

            let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

            for (offset, window) in data.windows(target_bytes.len()).enumerate() {
                if window == target_bytes {
                    results.push(Address::new(region.range.start.as_u64() + offset as u64));
                }
            }
        }

        Ok(results)
    }

    pub fn search_for_string_case_insensitive(&self, target: &str, regions: &[MemoryRegion]) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();
        let target_lower = target.to_lowercase();
        let target_bytes = target_lower.as_bytes();

        for region in regions {
            if !region.protection.is_readable() {
                continue;
            }

            let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

            for offset in 0..data.len().saturating_sub(target_bytes.len() - 1) {
                let window = &data[offset..offset + target_bytes.len()];
                let window_lower: Vec<u8> = window.iter().map(|b| b.to_ascii_lowercase()).collect();

                if window_lower == target_bytes {
                    results.push(Address::new(region.range.start.as_u64() + offset as u64));
                }
            }
        }

        Ok(results)
    }

    pub fn categorize_strings(&self, strings: &[FoundString]) -> StringCategories {
        let mut categories = StringCategories::new();

        for string in strings {
            if self.looks_like_function_name(&string.content) {
                categories.function_names.push(string.clone());
            } else if self.looks_like_class_name(&string.content) {
                categories.class_names.push(string.clone());
            } else if self.looks_like_error_message(&string.content) {
                categories.error_messages.push(string.clone());
            } else if self.looks_like_path(&string.content) {
                categories.paths.push(string.clone());
            } else if self.looks_like_url(&string.content) {
                categories.urls.push(string.clone());
            } else {
                categories.other.push(string.clone());
            }
        }

        categories
    }

    fn is_printable_ascii(&self, byte: u8) -> bool {
        byte >= 0x20 && byte < 0x7F
    }

    fn is_valid_utf8_byte(&self, byte: u8, current: &[u8]) -> bool {
        if byte < 0x80 {
            return byte >= 0x20;
        }

        if current.is_empty() {
            return byte >= 0xC2 && byte < 0xF5;
        }

        byte >= 0x80 && byte < 0xC0
    }

    fn is_printable_wide(&self, wide_char: u16) -> bool {
        wide_char >= 0x20 && wide_char < 0xFFFF && wide_char != 0xFFFD
    }

    fn looks_like_function_name(&self, s: &str) -> bool {
        if s.is_empty() || s.len() > 200 {
            return false;
        }

        let has_valid_start = s.chars().next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false);

        let all_valid = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        let has_camel_case = s.chars().zip(s.chars().skip(1))
            .any(|(a, b)| a.is_ascii_lowercase() && b.is_ascii_uppercase());

        let has_underscore = s.contains('_');

        has_valid_start && all_valid && (has_camel_case || has_underscore)
    }

    fn looks_like_class_name(&self, s: &str) -> bool {
        if s.is_empty() || s.len() > 100 {
            return false;
        }

        let starts_upper = s.chars().next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false);

        let all_valid = s.chars().all(|c| c.is_ascii_alphanumeric());

        starts_upper && all_valid && !s.chars().all(|c| c.is_ascii_uppercase())
    }

    fn looks_like_error_message(&self, s: &str) -> bool {
        let lower = s.to_lowercase();

        lower.contains("error") ||
        lower.contains("fail") ||
        lower.contains("invalid") ||
        lower.contains("exception") ||
        lower.contains("cannot") ||
        lower.contains("unable to")
    }

    fn looks_like_path(&self, s: &str) -> bool {
        s.starts_with('/') ||
        s.starts_with("C:\\") ||
        s.contains("\\") && s.contains('.') ||
        s.contains('/') && s.contains('.')
    }

    fn looks_like_url(&self, s: &str) -> bool {
        s.starts_with("http://") ||
        s.starts_with("https://") ||
        s.starts_with("ftp://") ||
        s.starts_with("file://")
    }
}

#[derive(Debug, Clone)]
pub struct FoundString {
    pub address: Address,
    pub content: String,
    pub encoding: StringEncoding,
    pub is_null_terminated: bool,
}

impl FoundString {
    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Ascii,
    Utf8,
    Utf16Le,
    Utf16Be,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct StringCategories {
    pub function_names: Vec<FoundString>,
    pub class_names: Vec<FoundString>,
    pub error_messages: Vec<FoundString>,
    pub paths: Vec<FoundString>,
    pub urls: Vec<FoundString>,
    pub other: Vec<FoundString>,
}

impl StringCategories {
    pub fn new() -> Self {
        Self {
            function_names: Vec::new(),
            class_names: Vec::new(),
            error_messages: Vec::new(),
            paths: Vec::new(),
            urls: Vec::new(),
            other: Vec::new(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.function_names.len() +
        self.class_names.len() +
        self.error_messages.len() +
        self.paths.len() +
        self.urls.len() +
        self.other.len()
    }

    pub fn all(&self) -> Vec<&FoundString> {
        self.function_names.iter()
            .chain(self.class_names.iter())
            .chain(self.error_messages.iter())
            .chain(self.paths.iter())
            .chain(self.urls.iter())
            .chain(self.other.iter())
            .collect()
    }
}

impl Default for StringCategories {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StringIndex {
    by_content: HashMap<String, Vec<Address>>,
    by_address: HashMap<u64, FoundString>,
}

impl StringIndex {
    pub fn new() -> Self {
        Self {
            by_content: HashMap::new(),
            by_address: HashMap::new(),
        }
    }

    pub fn add(&mut self, string: FoundString) {
        self.by_content
            .entry(string.content.clone())
            .or_default()
            .push(string.address);

        self.by_address.insert(string.address.as_u64(), string);
    }

    pub fn find_by_content(&self, content: &str) -> Option<&Vec<Address>> {
        self.by_content.get(content)
    }

    pub fn find_by_address(&self, addr: Address) -> Option<&FoundString> {
        self.by_address.get(&addr.as_u64())
    }

    pub fn find_by_prefix(&self, prefix: &str) -> Vec<&FoundString> {
        self.by_address.values()
            .filter(|s| s.content.starts_with(prefix))
            .collect()
    }

    pub fn find_by_contains(&self, substring: &str) -> Vec<&FoundString> {
        self.by_address.values()
            .filter(|s| s.content.contains(substring))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.by_address.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_address.is_empty()
    }

    pub fn unique_strings(&self) -> usize {
        self.by_content.len()
    }
}

impl Default for StringIndex {
    fn default() -> Self {
        Self::new()
    }
}
