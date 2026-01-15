// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;

pub struct ValidationChecker {
    reader: Arc<dyn MemoryReader>,
}

impl ValidationChecker {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn check_address_readable(&self, addr: Address) -> Result<bool, MemoryError> {
        match self.reader.read_u8(addr) {
            Ok(_) => Ok(true),
            Err(MemoryError::ReadFailed(_, _)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn check_address_in_range(&self, addr: Address, min: u64, max: u64) -> bool {
        let a = addr.as_u64();
        a >= min && a <= max
    }

    pub fn check_alignment(&self, addr: Address, alignment: u64) -> bool {
        addr.as_u64() % alignment == 0
    }

    pub fn check_function_prologue(&self, addr: Address) -> Result<bool, MemoryError> {
        let bytes = self.reader.read_bytes(addr, 8)?;

        let inst0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let inst1 = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if (inst0 & 0xFFC003E0) == 0xA9800000 {
            return Ok(true);
        }

        if (inst0 & 0xFF0003E0) == 0xD10003E0 {
            return Ok(true);
        }

        if (inst0 & 0x9F000000) == 0x10000000 {
            return Ok(true);
        }

        if inst0 == 0xD503237F {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn check_vtable_pointer(&self, addr: Address) -> Result<bool, MemoryError> {
        let ptr = self.reader.read_u64(addr)?;

        if ptr == 0 {
            return Ok(false);
        }

        if ptr % 8 != 0 {
            return Ok(false);
        }

        if ptr < 0x100000000 || ptr > 0x800000000000 {
            return Ok(false);
        }

        let first_entry = self.reader.read_u64(Address::new(ptr))?;
        if first_entry != 0 && first_entry % 4 == 0 {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn check_string_pointer(&self, addr: Address) -> Result<Option<String>, MemoryError> {
        let ptr = self.reader.read_u64(addr)?;

        if ptr == 0 {
            return Ok(None);
        }

        let str_addr = Address::new(ptr);

        let mut bytes = Vec::new();
        let max_len = 256;

        for i in 0..max_len {
            let b = self.reader.read_u8(str_addr + i as u64)?;
            if b == 0 {
                break;
            }
            if !b.is_ascii() {
                return Ok(None);
            }
            bytes.push(b);
        }

        if bytes.is_empty() {
            return Ok(None);
        }

        Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
    }

    pub fn check_lua_string(&self, addr: Address) -> Result<Option<String>, MemoryError> {
        let len = self.reader.read_u32(addr + 0x10)? as usize;

        if len == 0 || len > 0x100000 {
            return Ok(None);
        }

        let data = self.reader.read_bytes(addr + 0x18, len)?;
        Ok(Some(String::from_utf8_lossy(&data).to_string()))
    }

    pub fn check_table_structure(&self, addr: Address) -> Result<bool, MemoryError> {
        let flags = self.reader.read_u8(addr + 0x08)?;
        let log2_size = self.reader.read_u8(addr + 0x09)?;

        if log2_size > 30 {
            return Ok(false);
        }

        let array_ptr = self.reader.read_u64(addr + 0x18)?;
        let node_ptr = self.reader.read_u64(addr + 0x20)?;

        if array_ptr != 0 && array_ptr % 8 != 0 {
            return Ok(false);
        }

        if node_ptr != 0 && node_ptr % 8 != 0 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_closure_structure(&self, addr: Address) -> Result<bool, MemoryError> {
        let is_c = self.reader.read_u8(addr + 0x08)?;
        let nupvalues = self.reader.read_u8(addr + 0x09)?;

        if is_c > 1 {
            return Ok(false);
        }

        if nupvalues > 100 {
            return Ok(false);
        }

        let env_ptr = self.reader.read_u64(addr + 0x18)?;
        if env_ptr != 0 && env_ptr % 8 != 0 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_proto_structure(&self, addr: Address) -> Result<bool, MemoryError> {
        let nups = self.reader.read_u8(addr + 0x08)?;
        let numparams = self.reader.read_u8(addr + 0x09)?;
        let maxstacksize = self.reader.read_u8(addr + 0x0B)?;

        if nups > 200 || numparams > 200 || maxstacksize > 250 {
            return Ok(false);
        }

        let sizecode = self.reader.read_u32(addr + 0x10)?;
        if sizecode == 0 || sizecode > 0x1000000 {
            return Ok(false);
        }

        let code_ptr = self.reader.read_u64(addr + 0x20)?;
        if code_ptr == 0 || code_ptr % 4 != 0 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn validate_memory_region(&self, start: Address, size: usize) -> Result<ValidationResult, MemoryError> {
        let mut result = ValidationResult::new();

        match self.reader.read_bytes(start, size.min(16)) {
            Ok(_) => result.is_readable = true,
            Err(_) => result.is_readable = false,
        }

        result.start = start;
        result.size = size;

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub start: Address,
    pub size: usize,
    pub is_readable: bool,
    pub is_executable: bool,
    pub contains_code: bool,
    pub contains_data: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            start: Address::new(0),
            size: 0,
            is_readable: false,
            is_executable: false,
            contains_code: false,
            contains_data: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_readable
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}
