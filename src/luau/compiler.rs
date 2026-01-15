// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;

pub struct CompilerInfo {
    pub compile_function: Option<Address>,
    pub encoder_key: Option<u8>,
    pub version: Option<u32>,
    pub optimization_level: OptimizationLevel,
    pub debug_level: DebugLevel,
}

impl CompilerInfo {
    pub fn new() -> Self {
        Self {
            compile_function: None,
            encoder_key: None,
            version: None,
            optimization_level: OptimizationLevel::Unknown,
            debug_level: DebugLevel::Unknown,
        }
    }

    pub fn with_compile_function(mut self, addr: Address) -> Self {
        self.compile_function = Some(addr);
        self
    }

    pub fn with_encoder_key(mut self, key: u8) -> Self {
        self.encoder_key = Some(key);
        self
    }

    pub fn with_version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    pub fn is_encoded(&self) -> bool {
        self.encoder_key.is_some() && self.encoder_key != Some(0)
    }

    pub fn decode_bytecode(&self, data: &[u8]) -> Vec<u8> {
        let key = self.encoder_key.unwrap_or(0);
        if key == 0 {
            return data.to_vec();
        }

        data.iter().map(|b| b ^ key).collect()
    }
}

impl Default for CompilerInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Full,
    Aggressive,
    Unknown,
}

impl OptimizationLevel {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Basic,
            2 => OptimizationLevel::Full,
            3 => OptimizationLevel::Aggressive,
            _ => OptimizationLevel::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
    None,
    LineInfo,
    FullDebug,
    Unknown,
}

impl DebugLevel {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => DebugLevel::None,
            1 => DebugLevel::LineInfo,
            2 => DebugLevel::FullDebug,
            _ => DebugLevel::Unknown,
        }
    }
}

pub struct CompilerAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl CompilerAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze(&self) -> Result<CompilerInfo, MemoryError> {
        let mut info = CompilerInfo::new();

        if let Ok(Some(compile_addr)) = self.find_compile_function() {
            info.compile_function = Some(compile_addr);
        }

        if let Ok(Some(key)) = self.find_encoder_key() {
            info.encoder_key = Some(key);
        }

        if let Ok(Some(version)) = self.find_bytecode_version() {
            info.version = Some(version);
        }

        Ok(info)
    }

    fn find_compile_function(&self) -> Result<Option<Address>, MemoryError> {
        Ok(None)
    }

    fn find_encoder_key(&self) -> Result<Option<u8>, MemoryError> {
        Ok(None)
    }

    fn find_bytecode_version(&self) -> Result<Option<u32>, MemoryError> {
        Ok(None)
    }

    pub fn find_luau_compile(&self) -> Result<Option<Address>, MemoryError> {
        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.protection().is_executable() {
                continue;
            }

            let data = self.reader.read_bytes(region.range().start(), region.range().size() as usize)?;

            for offset in (0..data.len().saturating_sub(16)).step_by(4) {
                let word = u32::from_le_bytes([
                    data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
                ]);

                let is_prologue = (word & 0xFFC07FFF) == 0xA9007BFD;
                if is_prologue {
                    let addr = Address::new(region.range().start().as_u64() + offset as u64);
                    if self.validate_compile_function(addr).unwrap_or(false) {
                        return Ok(Some(addr));
                    }
                }
            }
        }

        Ok(None)
    }

    fn validate_compile_function(&self, addr: Address) -> Result<bool, MemoryError> {
        Ok(false)
    }
}

pub struct BytecodeEncoder {
    key: u8,
}

impl BytecodeEncoder {
    pub fn new(key: u8) -> Self {
        Self { key }
    }

    pub fn from_key(key: u8) -> Self {
        Self { key }
    }

    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b ^ self.key).collect()
    }

    pub fn decode(&self, data: &[u8]) -> Vec<u8> {
        self.encode(data)
    }

    pub fn key(&self) -> u8 {
        self.key
    }

    pub fn is_active(&self) -> bool {
        self.key != 0
    }
}

impl Default for BytecodeEncoder {
    fn default() -> Self {
        Self { key: 0 }
    }
}

pub struct CompilerOptions {
    pub optimization_level: OptimizationLevel,
    pub debug_level: DebugLevel,
    pub coverage_enabled: bool,
    pub type_info_enabled: bool,
    pub native_codegen: bool,
}

impl CompilerOptions {
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::Full,
            debug_level: DebugLevel::LineInfo,
            coverage_enabled: false,
            type_info_enabled: false,
            native_codegen: false,
        }
    }

    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn with_debug(mut self, level: DebugLevel) -> Self {
        self.debug_level = level;
        self
    }

    pub fn enable_coverage(mut self) -> Self {
        self.coverage_enabled = true;
        self
    }

    pub fn enable_type_info(mut self) -> Self {
        self.type_info_enabled = true;
        self
    }

    pub fn enable_native(mut self) -> Self {
        self.native_codegen = true;
        self
    }
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompilerStatistics {
    pub total_functions: usize,
    pub total_instructions: usize,
    pub total_constants: usize,
    pub total_strings: usize,
    pub total_closures: usize,
    pub max_stack_size: u8,
    pub max_upvalues: u8,
}

impl CompilerStatistics {
    pub fn new() -> Self {
        Self {
            total_functions: 0,
            total_instructions: 0,
            total_constants: 0,
            total_strings: 0,
            total_closures: 0,
            max_stack_size: 0,
            max_upvalues: 0,
        }
    }
}

impl Default for CompilerStatistics {
    fn default() -> Self {
        Self::new()
    }
}
