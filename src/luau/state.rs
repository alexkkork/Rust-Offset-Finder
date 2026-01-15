// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::types::{LuauType, TypeTag};
use std::sync::Arc;
use std::collections::HashMap;

pub struct StateAnalyzer {
    reader: Arc<dyn MemoryReader>,
    offsets: LuaStateOffsets,
}

impl StateAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            offsets: LuaStateOffsets::default(),
        }
    }

    pub fn with_offsets(mut self, offsets: LuaStateOffsets) -> Self {
        self.offsets = offsets;
        self
    }

    pub fn analyze_state(&self, state_addr: Address) -> Result<LuaStateInfo, MemoryError> {
        let mut info = LuaStateInfo::new(state_addr);

        let top_addr = self.reader.read_u64(state_addr + self.offsets.top)?;
        info.top = Address::new(top_addr);

        let stack_addr = self.reader.read_u64(state_addr + self.offsets.stack)?;
        info.stack_base = Address::new(stack_addr);

        if top_addr > stack_addr && top_addr != 0 && stack_addr != 0 {
            info.stack_size = ((top_addr - stack_addr) / 16) as usize;
        }

        let ci_addr = self.reader.read_u64(state_addr + self.offsets.ci)?;
        info.current_ci = Address::new(ci_addr);

        let base_ci_addr = self.reader.read_u64(state_addr + self.offsets.base_ci)?;
        info.base_ci = Address::new(base_ci_addr);

        if ci_addr > base_ci_addr && ci_addr != 0 && base_ci_addr != 0 {
            info.call_depth = ((ci_addr - base_ci_addr) / self.offsets.ci_size) as usize;
        }

        let global_addr = self.reader.read_u64(state_addr + self.offsets.global_state)?;
        info.global_state = Address::new(global_addr);

        if let Some(status_offset) = self.offsets.status {
            let status = self.reader.read_u8(state_addr + status_offset)?;
            info.status = ThreadStatus::from_u8(status);
        }

        Ok(info)
    }

    pub fn read_stack_value(&self, state_addr: Address, index: i32) -> Result<StackValue, MemoryError> {
        let top_addr = self.reader.read_u64(state_addr + self.offsets.top)?;
        let stack_addr = self.reader.read_u64(state_addr + self.offsets.stack)?;

        let value_addr = if index >= 0 {
            stack_addr + (index as u64 * 16)
        } else {
            top_addr.saturating_add((index as i64 * 16) as u64)
        };

        self.read_tvalue(Address::new(value_addr))
    }

    pub fn read_tvalue(&self, addr: Address) -> Result<StackValue, MemoryError> {
        let data = self.reader.read_bytes(addr, 16)?;

        let tt = data[8];
        let type_tag = TypeTag::from_u8(tt);

        let value = match type_tag {
            TypeTag::Nil => StackValue::Nil,
            TypeTag::Boolean => {
                let b = data[0] != 0;
                StackValue::Boolean(b)
            }
            TypeTag::Number => {
                let n = f64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::Number(n)
            }
            TypeTag::LightUserData => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::LightUserData(Address::new(ptr))
            }
            TypeTag::String => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::String(Address::new(ptr))
            }
            TypeTag::Table => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::Table(Address::new(ptr))
            }
            TypeTag::Function => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::Function(Address::new(ptr))
            }
            TypeTag::UserData => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::UserData(Address::new(ptr))
            }
            TypeTag::Thread => {
                let ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                StackValue::Thread(Address::new(ptr))
            }
            TypeTag::Vector => {
                let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let z = f32::from_le_bytes([data[12], data[13], data[14], data[15]]);
                StackValue::Vector(x, y, z)
            }
            _ => StackValue::Unknown(tt),
        };

        Ok(value)
    }

    pub fn read_string_value(&self, str_addr: Address) -> Result<String, MemoryError> {
        let len = self.reader.read_u32(str_addr + 0x10)? as usize;

        if len == 0 {
            return Ok(String::new());
        }

        if len > 0x10000 {
            return Err(MemoryError::InvalidSize(len));
        }

        let data_addr = str_addr + 0x18;
        let bytes = self.reader.read_bytes(data_addr, len)?;

        String::from_utf8(bytes)
            .map_err(|_| MemoryError::InvalidString)
    }

    pub fn get_global_state_info(&self, state_addr: Address) -> Result<GlobalStateInfo, MemoryError> {
        let global_addr_val = self.reader.read_u64(state_addr + self.offsets.global_state)?;
        let global_addr = Address::new(global_addr_val);

        let mut info = GlobalStateInfo::new(global_addr);

        Ok(info)
    }

    pub fn find_all_threads(&self, main_state: Address) -> Result<Vec<Address>, MemoryError> {
        let mut threads = Vec::new();
        threads.push(main_state);

        Ok(threads)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LuaStateOffsets {
    pub top: u64,
    pub stack: u64,
    pub stack_last: u64,
    pub ci: u64,
    pub base_ci: u64,
    pub global_state: u64,
    pub ci_size: u64,
    pub status: Option<u64>,
    pub gclist: Option<u64>,
    pub tt: Option<u64>,
}

impl Default for LuaStateOffsets {
    fn default() -> Self {
        Self {
            top: 0x10,
            stack: 0x18,
            stack_last: 0x20,
            ci: 0x28,
            base_ci: 0x30,
            global_state: 0x38,
            ci_size: 0x28,
            status: Some(0x06),
            gclist: Some(0x00),
            tt: Some(0x08),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LuaStateInfo {
    pub address: Address,
    pub top: Address,
    pub stack_base: Address,
    pub stack_size: usize,
    pub current_ci: Address,
    pub base_ci: Address,
    pub call_depth: usize,
    pub global_state: Address,
    pub status: ThreadStatus,
}

impl LuaStateInfo {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            top: Address::new(0),
            stack_base: Address::new(0),
            stack_size: 0,
            current_ci: Address::new(0),
            base_ci: Address::new(0),
            call_depth: 0,
            global_state: Address::new(0),
            status: ThreadStatus::Unknown,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.top.as_u64() != 0 &&
        self.stack_base.as_u64() != 0 &&
        self.global_state.as_u64() != 0
    }

    pub fn is_running(&self) -> bool {
        self.status == ThreadStatus::Running
    }

    pub fn is_yielded(&self) -> bool {
        self.status == ThreadStatus::Yield
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Ok,
    Yield,
    ErrRun,
    ErrSyntax,
    ErrMem,
    ErrGcmm,
    ErrErr,
    Running,
    Suspended,
    Dead,
    Unknown,
}

impl ThreadStatus {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ThreadStatus::Ok,
            1 => ThreadStatus::Yield,
            2 => ThreadStatus::ErrRun,
            3 => ThreadStatus::ErrSyntax,
            4 => ThreadStatus::ErrMem,
            5 => ThreadStatus::ErrGcmm,
            6 => ThreadStatus::ErrErr,
            _ => ThreadStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StackValue {
    Nil,
    Boolean(bool),
    Number(f64),
    LightUserData(Address),
    String(Address),
    Table(Address),
    Function(Address),
    UserData(Address),
    Thread(Address),
    Vector(f32, f32, f32),
    Unknown(u8),
}

impl StackValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            StackValue::Nil => "nil",
            StackValue::Boolean(_) => "boolean",
            StackValue::Number(_) => "number",
            StackValue::LightUserData(_) => "lightuserdata",
            StackValue::String(_) => "string",
            StackValue::Table(_) => "table",
            StackValue::Function(_) => "function",
            StackValue::UserData(_) => "userdata",
            StackValue::Thread(_) => "thread",
            StackValue::Vector(_, _, _) => "vector",
            StackValue::Unknown(_) => "unknown",
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, StackValue::Nil)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, StackValue::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, StackValue::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, StackValue::String(_))
    }

    pub fn is_table(&self) -> bool {
        matches!(self, StackValue::Table(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, StackValue::Function(_))
    }

    pub fn is_gc_object(&self) -> bool {
        matches!(self,
            StackValue::String(_) |
            StackValue::Table(_) |
            StackValue::Function(_) |
            StackValue::UserData(_) |
            StackValue::Thread(_)
        )
    }
}

#[derive(Debug, Clone)]
pub struct GlobalStateInfo {
    pub address: Address,
    pub main_thread: Address,
    pub string_table: Address,
    pub gc_state: u8,
    pub total_bytes: u64,
}

impl GlobalStateInfo {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            main_thread: Address::new(0),
            string_table: Address::new(0),
            gc_state: 0,
            total_bytes: 0,
        }
    }
}
