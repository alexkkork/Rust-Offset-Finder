// Tue Jan 13 2026 - Alex

use crate::analysis::arm64::Register;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    Register(Register),
    Immediate(i64),
    PCRelative(i32),
    Memory {
        base: Option<Register>,
        index: Option<Register>,
        offset: i64,
        scale: u8,
        pre_index: bool,
        post_index: bool,
    },
    SystemRegister(u16),
    Condition(u8),
    Shifted {
        reg: Register,
        shift_type: ShiftType,
        amount: u8,
    },
    Extended {
        reg: Register,
        extend_type: ExtendType,
        amount: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendType {
    UXTB,
    UXTH,
    UXTW,
    UXTX,
    SXTB,
    SXTH,
    SXTW,
    SXTX,
    LSL,
}

#[derive(Debug, Clone)]
pub struct Operand {
    pub op_type: OperandType,
    pub size: u8,
}

impl Operand {
    pub fn register(reg: Register) -> Self {
        Self {
            op_type: OperandType::Register(reg),
            size: reg.size().bytes(),
        }
    }

    pub fn immediate(value: i64) -> Self {
        Self {
            op_type: OperandType::Immediate(value),
            size: 8,
        }
    }

    pub fn immediate_shifted(value: i64, shift_type: ShiftType, amount: u8) -> Self {
        Self {
            op_type: OperandType::Immediate(value << amount),
            size: 8,
        }
    }

    pub fn pc_relative(offset: i32) -> Self {
        Self {
            op_type: OperandType::PCRelative(offset),
            size: 8,
        }
    }

    pub fn memory_base(base: Register) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: None,
                offset: 0,
                scale: 1,
                pre_index: false,
                post_index: false,
            },
            size: 8,
        }
    }

    pub fn memory_offset(base: Register, offset: i64) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: None,
                offset,
                scale: 1,
                pre_index: false,
                post_index: false,
            },
            size: 8,
        }
    }

    pub fn memory_pre_index(base: Register, offset: i64) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: None,
                offset,
                scale: 1,
                pre_index: true,
                post_index: false,
            },
            size: 8,
        }
    }

    pub fn memory_post_index(base: Register, offset: i64) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: None,
                offset,
                scale: 1,
                pre_index: false,
                post_index: true,
            },
            size: 8,
        }
    }

    pub fn memory_indexed(base: Register, index: Register, scale: u8) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: Some(index),
                offset: 0,
                scale,
                pre_index: false,
                post_index: false,
            },
            size: 8,
        }
    }

    pub fn memory_indexed_extended(base: Register, index: Register, extend: ExtendType, amount: u8) -> Self {
        Self {
            op_type: OperandType::Memory {
                base: Some(base),
                index: Some(index),
                offset: 0,
                scale: 1 << amount,
                pre_index: false,
                post_index: false,
            },
            size: 8,
        }
    }

    pub fn system_register(sysreg: u16) -> Self {
        Self {
            op_type: OperandType::SystemRegister(sysreg),
            size: 8,
        }
    }

    pub fn register_shifted(reg: Register, shift_type: ShiftType, amount: u8) -> Self {
        Self {
            op_type: OperandType::Shifted {
                reg,
                shift_type,
                amount,
            },
            size: reg.size().bytes(),
        }
    }

    pub fn register_extended(reg: Register, extend_type: ExtendType, amount: u8) -> Self {
        Self {
            op_type: OperandType::Extended {
                reg,
                extend_type,
                amount,
            },
            size: reg.size().bytes(),
        }
    }

    pub fn is_register(&self) -> bool {
        matches!(self.op_type, OperandType::Register(_))
    }

    pub fn is_immediate(&self) -> bool {
        matches!(self.op_type, OperandType::Immediate(_))
    }

    pub fn is_memory(&self) -> bool {
        matches!(self.op_type, OperandType::Memory { .. })
    }

    pub fn is_pc_relative(&self) -> bool {
        matches!(self.op_type, OperandType::PCRelative(_))
    }

    pub fn get_register(&self) -> Option<Register> {
        match self.op_type {
            OperandType::Register(r) => Some(r),
            OperandType::Shifted { reg, .. } => Some(reg),
            OperandType::Extended { reg, .. } => Some(reg),
            _ => None,
        }
    }

    pub fn get_immediate(&self) -> Option<i64> {
        match self.op_type {
            OperandType::Immediate(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_pc_relative(&self) -> Option<i32> {
        match self.op_type {
            OperandType::PCRelative(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_memory_base(&self) -> Option<Register> {
        match self.op_type {
            OperandType::Memory { base, .. } => base,
            _ => None,
        }
    }

    pub fn get_memory_offset(&self) -> Option<i64> {
        match self.op_type {
            OperandType::Memory { offset, .. } => Some(offset),
            _ => None,
        }
    }

    pub fn get_memory_index(&self) -> Option<Register> {
        match self.op_type {
            OperandType::Memory { index, .. } => index,
            _ => None,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.op_type {
            OperandType::Register(reg) => write!(f, "{}", reg),
            OperandType::Immediate(imm) => {
                if *imm < 0 {
                    write!(f, "#-{:#x}", (-imm) as u64)
                } else {
                    write!(f, "#{:#x}", *imm as u64)
                }
            }
            OperandType::PCRelative(offset) => {
                if *offset < 0 {
                    write!(f, ".-{:#x}", (-offset) as u32)
                } else {
                    write!(f, ".+{:#x}", *offset as u32)
                }
            }
            OperandType::Memory { base, index, offset, scale, pre_index, post_index } => {
                write!(f, "[")?;
                if let Some(b) = base {
                    write!(f, "{}", b)?;
                }
                if let Some(idx) = index {
                    write!(f, ", {}", idx)?;
                    if *scale > 1 {
                        write!(f, ", lsl #{}", (*scale as u32).trailing_zeros())?;
                    }
                } else if *offset != 0 {
                    if *pre_index {
                        write!(f, ", #{}]!", offset)?;
                        return Ok(());
                    } else {
                        write!(f, ", #{}", offset)?;
                    }
                }
                write!(f, "]")?;
                if *post_index && *offset != 0 {
                    write!(f, ", #{}", offset)?;
                }
                Ok(())
            }
            OperandType::SystemRegister(sysreg) => write!(f, "s{}", sysreg),
            OperandType::Condition(cond) => {
                let cond_str = match cond {
                    0 => "eq", 1 => "ne", 2 => "cs", 3 => "cc",
                    4 => "mi", 5 => "pl", 6 => "vs", 7 => "vc",
                    8 => "hi", 9 => "ls", 10 => "ge", 11 => "lt",
                    12 => "gt", 13 => "le", 14 => "al", _ => "nv",
                };
                write!(f, "{}", cond_str)
            }
            OperandType::Shifted { reg, shift_type, amount } => {
                write!(f, "{}", reg)?;
                if *amount > 0 {
                    let shift_str = match shift_type {
                        ShiftType::LSL => "lsl",
                        ShiftType::LSR => "lsr",
                        ShiftType::ASR => "asr",
                        ShiftType::ROR => "ror",
                    };
                    write!(f, ", {} #{}", shift_str, amount)?;
                }
                Ok(())
            }
            OperandType::Extended { reg, extend_type, amount } => {
                let ext_str = match extend_type {
                    ExtendType::UXTB => "uxtb",
                    ExtendType::UXTH => "uxth",
                    ExtendType::UXTW => "uxtw",
                    ExtendType::UXTX => "uxtx",
                    ExtendType::SXTB => "sxtb",
                    ExtendType::SXTH => "sxth",
                    ExtendType::SXTW => "sxtw",
                    ExtendType::SXTX => "sxtx",
                    ExtendType::LSL => "lsl",
                };
                write!(f, "{}, {}", reg, ext_str)?;
                if *amount > 0 {
                    write!(f, " #{}", amount)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for ShiftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShiftType::LSL => write!(f, "lsl"),
            ShiftType::LSR => write!(f, "lsr"),
            ShiftType::ASR => write!(f, "asr"),
            ShiftType::ROR => write!(f, "ror"),
        }
    }
}

impl fmt::Display for ExtendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtendType::UXTB => write!(f, "uxtb"),
            ExtendType::UXTH => write!(f, "uxth"),
            ExtendType::UXTW => write!(f, "uxtw"),
            ExtendType::UXTX => write!(f, "uxtx"),
            ExtendType::SXTB => write!(f, "sxtb"),
            ExtendType::SXTH => write!(f, "sxth"),
            ExtendType::SXTW => write!(f, "sxtw"),
            ExtendType::SXTX => write!(f, "sxtx"),
            ExtendType::LSL => write!(f, "lsl"),
        }
    }
}

pub fn parse_shift_type(s: &str) -> Option<ShiftType> {
    match s.to_lowercase().as_str() {
        "lsl" => Some(ShiftType::LSL),
        "lsr" => Some(ShiftType::LSR),
        "asr" => Some(ShiftType::ASR),
        "ror" => Some(ShiftType::ROR),
        _ => None,
    }
}

pub fn parse_extend_type(s: &str) -> Option<ExtendType> {
    match s.to_lowercase().as_str() {
        "uxtb" => Some(ExtendType::UXTB),
        "uxth" => Some(ExtendType::UXTH),
        "uxtw" | "lsl" => Some(ExtendType::UXTW),
        "uxtx" => Some(ExtendType::UXTX),
        "sxtb" => Some(ExtendType::SXTB),
        "sxth" => Some(ExtendType::SXTH),
        "sxtw" => Some(ExtendType::SXTW),
        "sxtx" => Some(ExtendType::SXTX),
        _ => None,
    }
}
