// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

pub struct PropertyAccessor {
    reader: Arc<dyn MemoryReader>,
}

impl PropertyAccessor {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze_getter(&self, addr: Address) -> Option<GetterInfo> {
        let bytes = self.reader.read_bytes(addr, 128).ok()?;

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return None;
        }

        let mut offset = None;
        let mut return_type = ReturnType::Unknown;
        let mut is_simple = false;

        for i in (0..bytes.len().min(64) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFC00000) == 0xF9400000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 8;
                offset = Some(imm12);
                return_type = ReturnType::Pointer;
            }

            if (insn & 0xFFC00000) == 0xB9400000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 4;
                offset = Some(imm12);
                return_type = ReturnType::Int32;
            }

            if (insn & 0xFFE00000) == 0xBD400000 {
                return_type = ReturnType::Float32;
            }

            if (insn & 0xFFE00000) == 0xFD400000 {
                return_type = ReturnType::Float64;
            }

            if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                if i < 32 {
                    is_simple = true;
                }
                break;
            }
        }

        Some(GetterInfo {
            address: addr,
            offset,
            return_type,
            is_simple,
        })
    }

    pub fn analyze_setter(&self, addr: Address) -> Option<SetterInfo> {
        let bytes = self.reader.read_bytes(addr, 128).ok()?;

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return None;
        }

        let mut offset = None;
        let mut value_type = ValueType::Unknown;
        let mut has_validation = false;
        let mut triggers_changed = false;

        for i in (0..bytes.len().min(96) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFC00000) == 0xF9000000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 8;
                offset = Some(imm12);
                value_type = ValueType::Pointer;
            }

            if (insn & 0xFFC00000) == 0xB9000000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 4;
                offset = Some(imm12);
                value_type = ValueType::Int32;
            }

            if (insn & 0xFF000000) == 0x54000000 {
                has_validation = true;
            }

            if (insn & 0xFC000000) == 0x94000000 {
                triggers_changed = true;
            }
        }

        Some(SetterInfo {
            address: addr,
            offset,
            value_type,
            has_validation,
            triggers_changed,
        })
    }

    pub fn infer_property_type(&self, getter: &GetterInfo, setter: Option<&SetterInfo>) -> PropertyType {
        match getter.return_type {
            ReturnType::Int32 => PropertyType::Integer,
            ReturnType::Float32 | ReturnType::Float64 => PropertyType::Number,
            ReturnType::Pointer => {
                if let Some(set) = setter {
                    if set.has_validation {
                        PropertyType::Instance
                    } else {
                        PropertyType::Unknown
                    }
                } else {
                    PropertyType::Unknown
                }
            }
            ReturnType::Unknown => PropertyType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetterInfo {
    pub address: Address,
    pub offset: Option<u64>,
    pub return_type: ReturnType,
    pub is_simple: bool,
}

#[derive(Debug, Clone)]
pub struct SetterInfo {
    pub address: Address,
    pub offset: Option<u64>,
    pub value_type: ValueType,
    pub has_validation: bool,
    pub triggers_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
    Int32,
    Int64,
    Float32,
    Float64,
    Pointer,
    Void,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int32,
    Int64,
    Float32,
    Float64,
    Pointer,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Integer,
    Number,
    Boolean,
    String,
    Instance,
    Vector3,
    CFrame,
    Color3,
    UDim2,
    Enum,
    Unknown,
}
