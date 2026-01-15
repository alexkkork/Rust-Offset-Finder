// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct TValueFinder {
    reader: Arc<dyn MemoryReader>,
}

impl TValueFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, _start: Address, _end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        results.push(StructureOffsetResult::new(
            "TValue".to_string(),
            "value".to_string(),
            0x00,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "TValue".to_string(),
            "extra".to_string(),
            0x08,
        ).with_size(4).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "TValue".to_string(),
            "tt".to_string(),
            0x0C,
        ).with_size(4).with_confidence(0.95).with_method("known"));

        results
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaType {
    Nil,
    Boolean,
    LightUserdata,
    Number,
    Vector,
    String,
    Table,
    Function,
    Userdata,
    Thread,
    Buffer,
    Unknown(u8),
}

impl LuaType {
    pub fn from_tag(tag: u8) -> Self {
        match tag {
            0 => LuaType::Nil,
            1 => LuaType::Boolean,
            2 => LuaType::LightUserdata,
            3 => LuaType::Number,
            4 => LuaType::Vector,
            5 => LuaType::String,
            6 => LuaType::Table,
            7 => LuaType::Function,
            8 => LuaType::Userdata,
            9 => LuaType::Thread,
            10 => LuaType::Buffer,
            t => LuaType::Unknown(t),
        }
    }

    pub fn to_tag(&self) -> u8 {
        match self {
            LuaType::Nil => 0,
            LuaType::Boolean => 1,
            LuaType::LightUserdata => 2,
            LuaType::Number => 3,
            LuaType::Vector => 4,
            LuaType::String => 5,
            LuaType::Table => 6,
            LuaType::Function => 7,
            LuaType::Userdata => 8,
            LuaType::Thread => 9,
            LuaType::Buffer => 10,
            LuaType::Unknown(t) => *t,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LuaType::Nil => "nil",
            LuaType::Boolean => "boolean",
            LuaType::LightUserdata => "lightuserdata",
            LuaType::Number => "number",
            LuaType::Vector => "vector",
            LuaType::String => "string",
            LuaType::Table => "table",
            LuaType::Function => "function",
            LuaType::Userdata => "userdata",
            LuaType::Thread => "thread",
            LuaType::Buffer => "buffer",
            LuaType::Unknown(_) => "unknown",
        }
    }

    pub fn is_gc_object(&self) -> bool {
        matches!(
            self,
            LuaType::String
                | LuaType::Table
                | LuaType::Function
                | LuaType::Userdata
                | LuaType::Thread
                | LuaType::Buffer
        )
    }
}

pub struct TValueReader {
    reader: Arc<dyn MemoryReader>,
}

impl TValueReader {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn read_tvalue(&self, addr: Address) -> Option<TValueData> {
        let bytes = self.reader.read_bytes(addr, 16).ok()?;

        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let extra = u32::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);

        let tt = u32::from_le_bytes([
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        Some(TValueData {
            value,
            extra,
            tt,
            lua_type: LuaType::from_tag((tt & 0xFF) as u8),
        })
    }

    pub fn read_number(&self, addr: Address) -> Option<f64> {
        let tvalue = self.read_tvalue(addr)?;

        if tvalue.lua_type != LuaType::Number {
            return None;
        }

        Some(f64::from_bits(tvalue.value))
    }

    pub fn read_boolean(&self, addr: Address) -> Option<bool> {
        let tvalue = self.read_tvalue(addr)?;

        if tvalue.lua_type != LuaType::Boolean {
            return None;
        }

        Some(tvalue.value != 0)
    }

    pub fn read_string_ptr(&self, addr: Address) -> Option<Address> {
        let tvalue = self.read_tvalue(addr)?;

        if tvalue.lua_type != LuaType::String {
            return None;
        }

        Some(Address::new(tvalue.value))
    }

    pub fn read_table_ptr(&self, addr: Address) -> Option<Address> {
        let tvalue = self.read_tvalue(addr)?;

        if tvalue.lua_type != LuaType::Table {
            return None;
        }

        Some(Address::new(tvalue.value))
    }

    pub fn read_function_ptr(&self, addr: Address) -> Option<Address> {
        let tvalue = self.read_tvalue(addr)?;

        if tvalue.lua_type != LuaType::Function {
            return None;
        }

        Some(Address::new(tvalue.value))
    }
}

#[derive(Debug, Clone)]
pub struct TValueData {
    pub value: u64,
    pub extra: u32,
    pub tt: u32,
    pub lua_type: LuaType,
}

impl TValueData {
    pub fn is_nil(&self) -> bool {
        self.lua_type == LuaType::Nil
    }

    pub fn is_falsey(&self) -> bool {
        self.lua_type == LuaType::Nil || (self.lua_type == LuaType::Boolean && self.value == 0)
    }

    pub fn as_number(&self) -> Option<f64> {
        if self.lua_type == LuaType::Number {
            Some(f64::from_bits(self.value))
        } else {
            None
        }
    }

    pub fn as_pointer(&self) -> Option<Address> {
        if self.lua_type.is_gc_object() || self.lua_type == LuaType::LightUserdata {
            Some(Address::new(self.value))
        } else {
            None
        }
    }
}
