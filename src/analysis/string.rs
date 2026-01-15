// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError, MemoryRegion};
use std::sync::Arc;
use std::collections::HashMap;

pub type FoundString = StringInfo;

pub struct StringAnalyzer {
    reader: Arc<dyn MemoryReader>,
    config: StringAnalyzerConfig,
    string_cache: HashMap<u64, StringInfo>,
}

impl StringAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            config: StringAnalyzerConfig::default(),
            string_cache: HashMap::new(),
        }
    }

    pub fn with_config(reader: Arc<dyn MemoryReader>, config: StringAnalyzerConfig) -> Self {
        Self {
            reader,
            config,
            string_cache: HashMap::new(),
        }
    }

    pub fn find_strings_in_region(&self, region: &MemoryRegion) -> Result<Vec<FoundString>, MemoryError> {
        self.scan_region(region)
    }

    pub fn find_strings(&mut self, regions: &[MemoryRegion]) -> Result<Vec<StringInfo>, MemoryError> {
        let mut strings = Vec::new();

        for region in regions {
            if !region.protection.is_readable() {
                continue;
            }

            let region_strings = self.scan_region(region)?;
            strings.extend(region_strings);
        }

        for string in &strings {
            self.string_cache.insert(string.address.as_u64(), string.clone());
        }

        Ok(strings)
    }

    fn scan_region(&self, region: &MemoryRegion) -> Result<Vec<StringInfo>, MemoryError> {
        let mut strings = Vec::new();
        let start = region.range.start;
        let size = region.range.size() as usize;

        if size > self.config.max_region_size {
            return Ok(strings);
        }

        let data = self.reader.read_bytes(start, size)?;
        let mut i = 0;

        while i < data.len() {
            if self.is_printable_start(data[i]) {
                let (string, len) = self.extract_string(&data[i..]);

                if len >= self.config.min_length && len <= self.config.max_length {
                    let string_type = self.classify_string(&string);
                    let relevance = self.calculate_relevance(&string, string_type);

                    if relevance >= self.config.min_relevance {
                        strings.push(StringInfo {
                            address: start + i as u64,
                            content: string,
                            length: len,
                            string_type,
                            relevance,
                            encoding: StringEncoding::Ascii,
                            references: Vec::new(),
                        });
                    }

                    i += len;
                    continue;
                }
            }
            i += 1;
        }

        Ok(strings)
    }

    fn is_printable_start(&self, byte: u8) -> bool {
        byte >= 0x20 && byte < 0x7F
    }

    fn extract_string(&self, data: &[u8]) -> (String, usize) {
        let mut chars = Vec::new();
        let mut len = 0;

        for &byte in data {
            if byte == 0 || !self.is_valid_string_char(byte) {
                break;
            }
            chars.push(byte);
            len += 1;

            if len >= self.config.max_length {
                break;
            }
        }

        (String::from_utf8_lossy(&chars).to_string(), len)
    }

    fn is_valid_string_char(&self, byte: u8) -> bool {
        byte >= 0x20 && byte < 0x7F || byte == 0x09 || byte == 0x0A || byte == 0x0D
    }

    fn classify_string(&self, content: &str) -> StringType {
        let lower = content.to_lowercase();

        if lower.starts_with("lua_") || lower.starts_with("luau_") || lower.starts_with("lual_") {
            return StringType::LuaApi;
        }

        if lower.contains("rbx") || lower.contains("roblox") {
            return StringType::Roblox;
        }

        if lower.contains("script") || lower.contains("modulescript") || lower.contains("localscript") {
            return StringType::Script;
        }

        if lower.contains("instance") || lower.contains("part") || lower.contains("model") ||
           lower.contains("workspace") || lower.contains("player") {
            return StringType::Instance;
        }

        if lower.contains("error") || lower.contains("warning") || lower.contains("failed") {
            return StringType::Error;
        }

        if lower.contains("debug") || lower.contains("trace") || lower.contains("log") {
            return StringType::Debug;
        }

        if lower.ends_with(".lua") || lower.ends_with(".luau") {
            return StringType::SourceFile;
        }

        if lower.starts_with("http") || lower.contains("://") {
            return StringType::Url;
        }

        if content.chars().all(|c| c.is_alphanumeric() || c == '_') {
            if content.starts_with(|c: char| c.is_uppercase()) {
                return StringType::ClassName;
            }
            if content.starts_with(|c: char| c.is_lowercase()) {
                return StringType::Identifier;
            }
        }

        StringType::Unknown
    }

    fn calculate_relevance(&self, content: &str, string_type: StringType) -> f64 {
        let mut relevance = 0.5;

        match string_type {
            StringType::LuaApi => relevance += 0.4,
            StringType::Roblox => relevance += 0.35,
            StringType::Script => relevance += 0.3,
            StringType::Instance => relevance += 0.25,
            StringType::Error => relevance += 0.2,
            StringType::Debug => relevance += 0.15,
            StringType::SourceFile => relevance += 0.1,
            StringType::ClassName => relevance += 0.2,
            StringType::Identifier => relevance += 0.1,
            StringType::Url => relevance += 0.05,
            StringType::Unknown => {}
        }

        let len = content.len();
        if len >= 4 && len <= 64 {
            relevance += 0.1;
        }

        let keywords = ["state", "thread", "closure", "proto", "table", "stack", "identity", "capabilities"];
        let lower = content.to_lowercase();
        for keyword in keywords {
            if lower.contains(keyword) {
                relevance += 0.1;
                break;
            }
        }

        relevance.min(1.0)
    }

    pub fn find_references_to_string(&self, string_addr: Address, code_regions: &[MemoryRegion]) -> Result<Vec<StringReference>, MemoryError> {
        let mut references = Vec::new();

        for region in code_regions {
            if !region.protection.is_readable() {
                continue;
            }

            let refs = self.scan_for_references(string_addr, region)?;
            references.extend(refs);
        }

        Ok(references)
    }

    fn scan_for_references(&self, target: Address, region: &MemoryRegion) -> Result<Vec<StringReference>, MemoryError> {
        let mut refs = Vec::new();
        let start = region.range.start;
        let size = region.range.size() as usize;

        if size > self.config.max_region_size {
            return Ok(refs);
        }

        let data = self.reader.read_bytes(start, size)?;

        for i in (0..data.len().saturating_sub(8)).step_by(4) {
            let bytes = &data[i..i+4];
            let inst = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (inst >> 24) == 0x90 {
                let rd = inst & 0x1F;
                let immhi = ((inst >> 5) & 0x7FFFF) as i64;
                let immlo = ((inst >> 29) & 0x3) as i64;
                let page_offset = ((immhi << 2) | immlo) << 12;

                let current_addr = start + i as u64;
                let current_page = (current_addr.as_u64() as i64) & !0xFFF;
                let target_page = (current_page + page_offset) as u64;

                if i + 8 <= data.len() {
                    let next_bytes = &data[i+4..i+8];
                    let next_inst = u32::from_le_bytes([next_bytes[0], next_bytes[1], next_bytes[2], next_bytes[3]]);

                    if (next_inst >> 22) == 0x244 || (next_inst >> 24) == 0x91 {
                        let imm = if (next_inst >> 22) == 0x244 {
                            ((next_inst >> 10) & 0xFFF) * 8
                        } else {
                            (next_inst >> 10) & 0xFFF
                        };

                        let full_addr = target_page + imm as u64;

                        if full_addr == target.as_u64() {
                            refs.push(StringReference {
                                from_address: current_addr,
                                reference_type: ReferenceType::AdrpAdd,
                                instruction_count: 2,
                            });
                        }
                    }
                }
            }
        }

        Ok(refs)
    }

    pub fn find_function_strings(&self, func_addr: Address, func_size: usize) -> Result<Vec<(String, Address)>, MemoryError> {
        let mut found = Vec::new();
        let data = self.reader.read_bytes(func_addr, func_size)?;

        let mut string_addrs = Vec::new();

        for i in (0..data.len().saturating_sub(8)).step_by(4) {
            let bytes = &data[i..i+4];
            let inst = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (inst >> 24) == 0x90 {
                let immhi = ((inst >> 5) & 0x7FFFF) as i64;
                let immlo = ((inst >> 29) & 0x3) as i64;
                let page_offset = ((immhi << 2) | immlo) << 12;

                let current_addr = func_addr + i as u64;
                let current_page = (current_addr.as_u64() as i64) & !0xFFF;
                let target_page = (current_page + page_offset) as u64;

                if i + 8 <= data.len() {
                    let next_bytes = &data[i+4..i+8];
                    let next_inst = u32::from_le_bytes([next_bytes[0], next_bytes[1], next_bytes[2], next_bytes[3]]);

                    if (next_inst >> 24) == 0x91 {
                        let imm = (next_inst >> 10) & 0xFFF;
                        let full_addr = target_page + imm as u64;
                        string_addrs.push(Address::new(full_addr));
                    }
                }
            }
        }

        for addr in string_addrs {
            if let Ok(string_data) = self.reader.read_bytes(addr, 256) {
                let (content, len) = self.extract_string(&string_data);
                if len >= 2 && len <= 128 {
                    found.push((content, addr));
                }
            }
        }

        Ok(found)
    }

    pub fn get_cached_string(&self, addr: Address) -> Option<&StringInfo> {
        self.string_cache.get(&addr.as_u64())
    }

    pub fn search_strings(&self, pattern: &str) -> Vec<&StringInfo> {
        let pattern_lower = pattern.to_lowercase();
        self.string_cache.values()
            .filter(|s| s.content.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    pub fn strings_by_type(&self, string_type: StringType) -> Vec<&StringInfo> {
        self.string_cache.values()
            .filter(|s| s.string_type == string_type)
            .collect()
    }

    pub fn high_relevance_strings(&self, min_relevance: f64) -> Vec<&StringInfo> {
        self.string_cache.values()
            .filter(|s| s.relevance >= min_relevance)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.string_cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.string_cache.len()
    }
}

#[derive(Debug, Clone)]
pub struct StringInfo {
    pub address: Address,
    pub content: String,
    pub length: usize,
    pub string_type: StringType,
    pub relevance: f64,
    pub encoding: StringEncoding,
    pub references: Vec<StringReference>,
}

impl StringInfo {
    pub fn is_relevant(&self) -> bool {
        self.relevance >= 0.5
    }

    pub fn is_lua_related(&self) -> bool {
        matches!(self.string_type, StringType::LuaApi | StringType::Script | StringType::SourceFile)
    }

    pub fn is_roblox_related(&self) -> bool {
        matches!(self.string_type, StringType::Roblox | StringType::Instance | StringType::Script)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    Unknown,
    LuaApi,
    Roblox,
    Script,
    Instance,
    Error,
    Debug,
    SourceFile,
    Url,
    ClassName,
    Identifier,
}

impl StringType {
    pub fn name(&self) -> &'static str {
        match self {
            StringType::Unknown => "Unknown",
            StringType::LuaApi => "Lua API",
            StringType::Roblox => "Roblox",
            StringType::Script => "Script",
            StringType::Instance => "Instance",
            StringType::Error => "Error",
            StringType::Debug => "Debug",
            StringType::SourceFile => "Source File",
            StringType::Url => "URL",
            StringType::ClassName => "Class Name",
            StringType::Identifier => "Identifier",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Ascii,
    Utf8,
    Utf16Le,
    Utf16Be,
}

#[derive(Debug, Clone)]
pub struct StringReference {
    pub from_address: Address,
    pub reference_type: ReferenceType,
    pub instruction_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    Direct,
    AdrpAdd,
    AdrpLdr,
    Relative,
}

impl ReferenceType {
    pub fn name(&self) -> &'static str {
        match self {
            ReferenceType::Direct => "Direct",
            ReferenceType::AdrpAdd => "ADRP+ADD",
            ReferenceType::AdrpLdr => "ADRP+LDR",
            ReferenceType::Relative => "Relative",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringAnalyzerConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub min_relevance: f64,
    pub max_region_size: usize,
    pub scan_utf16: bool,
}

impl Default for StringAnalyzerConfig {
    fn default() -> Self {
        Self {
            min_length: 4,
            max_length: 512,
            min_relevance: 0.3,
            max_region_size: 0x10000000,
            scan_utf16: false,
        }
    }
}

pub struct StringSearcher {
    strings: Vec<StringInfo>,
}

impl StringSearcher {
    pub fn new(strings: Vec<StringInfo>) -> Self {
        Self { strings }
    }

    pub fn search_exact(&self, query: &str) -> Vec<&StringInfo> {
        self.strings.iter()
            .filter(|s| s.content == query)
            .collect()
    }

    pub fn search_contains(&self, query: &str) -> Vec<&StringInfo> {
        let query_lower = query.to_lowercase();
        self.strings.iter()
            .filter(|s| s.content.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn search_prefix(&self, prefix: &str) -> Vec<&StringInfo> {
        let prefix_lower = prefix.to_lowercase();
        self.strings.iter()
            .filter(|s| s.content.to_lowercase().starts_with(&prefix_lower))
            .collect()
    }

    pub fn search_suffix(&self, suffix: &str) -> Vec<&StringInfo> {
        let suffix_lower = suffix.to_lowercase();
        self.strings.iter()
            .filter(|s| s.content.to_lowercase().ends_with(&suffix_lower))
            .collect()
    }

    pub fn by_type(&self, string_type: StringType) -> Vec<&StringInfo> {
        self.strings.iter()
            .filter(|s| s.string_type == string_type)
            .collect()
    }

    pub fn by_relevance(&self, min: f64, max: f64) -> Vec<&StringInfo> {
        self.strings.iter()
            .filter(|s| s.relevance >= min && s.relevance <= max)
            .collect()
    }

    pub fn sorted_by_relevance(&self) -> Vec<&StringInfo> {
        let mut sorted: Vec<_> = self.strings.iter().collect();
        sorted.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    pub fn sorted_by_length(&self) -> Vec<&StringInfo> {
        let mut sorted: Vec<_> = self.strings.iter().collect();
        sorted.sort_by_key(|s| s.length);
        sorted
    }

    pub fn count(&self) -> usize {
        self.strings.len()
    }
}
