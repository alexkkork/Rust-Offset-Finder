// Tue Jan 13 2026 - Alex

use super::Register;

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory { base: u8, offset: i64 },
    Shift(u8),
    Extend(ExtendType, u8),
    Label(String),
    Condition(ConditionCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendType {
    Uxtb,
    Uxth,
    Uxtw,
    Uxtx,
    Sxtb,
    Sxth,
    Sxtw,
    Sxtx,
    Lsl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionCode {
    Eq = 0,
    Ne = 1,
    Cs = 2,
    Cc = 3,
    Mi = 4,
    Pl = 5,
    Vs = 6,
    Vc = 7,
    Hi = 8,
    Ls = 9,
    Ge = 10,
    Lt = 11,
    Gt = 12,
    Le = 13,
    Al = 14,
    Nv = 15,
}

impl Operand {
    pub fn reg(index: u8, is_64bit: bool) -> Self {
        Operand::Register(Register::new_gpr(index, is_64bit))
    }

    pub fn imm(value: i64) -> Self {
        Operand::Immediate(value)
    }

    pub fn mem(base: u8, offset: i64) -> Self {
        Operand::Memory { base, offset }
    }

    pub fn shift(amount: u8) -> Self {
        Operand::Shift(amount)
    }

    pub fn extend(ext_type: ExtendType, amount: u8) -> Self {
        Operand::Extend(ext_type, amount)
    }

    pub fn label(name: &str) -> Self {
        Operand::Label(name.to_string())
    }

    pub fn cond(code: ConditionCode) -> Self {
        Operand::Condition(code)
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Operand::Register(_))
    }

    pub fn is_immediate(&self) -> bool {
        matches!(self, Operand::Immediate(_))
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, Operand::Memory { .. })
    }

    pub fn as_register(&self) -> Option<&Register> {
        match self {
            Operand::Register(reg) => Some(reg),
            _ => None,
        }
    }

    pub fn as_immediate(&self) -> Option<i64> {
        match self {
            Operand::Immediate(imm) => Some(*imm),
            _ => None,
        }
    }

    pub fn as_memory(&self) -> Option<(u8, i64)> {
        match self {
            Operand::Memory { base, offset } => Some((*base, *offset)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(imm) => {
                if *imm >= 0 {
                    write!(f, "#0x{:x}", imm)
                } else {
                    write!(f, "#-0x{:x}", imm.unsigned_abs())
                }
            }
            Operand::Memory { base, offset } => {
                let base_name = if *base == 31 { "sp" } else { &format!("x{}", base) };
                if *offset == 0 {
                    write!(f, "[{}]", base_name)
                } else if *offset > 0 {
                    write!(f, "[{}, #0x{:x}]", base_name, offset)
                } else {
                    write!(f, "[{}, #-0x{:x}]", base_name, offset.unsigned_abs())
                }
            }
            Operand::Shift(amount) => write!(f, "lsl #{}", amount),
            Operand::Extend(ext_type, amount) => {
                let ext_str = match ext_type {
                    ExtendType::Uxtb => "uxtb",
                    ExtendType::Uxth => "uxth",
                    ExtendType::Uxtw => "uxtw",
                    ExtendType::Uxtx => "uxtx",
                    ExtendType::Sxtb => "sxtb",
                    ExtendType::Sxth => "sxth",
                    ExtendType::Sxtw => "sxtw",
                    ExtendType::Sxtx => "sxtx",
                    ExtendType::Lsl => "lsl",
                };
                if *amount == 0 {
                    write!(f, "{}", ext_str)
                } else {
                    write!(f, "{} #{}", ext_str, amount)
                }
            }
            Operand::Label(name) => write!(f, "{}", name),
            Operand::Condition(code) => write!(f, "{}", code),
        }
    }
}

impl ConditionCode {
    pub fn from_encoding(encoding: u8) -> Self {
        match encoding & 0xF {
            0 => ConditionCode::Eq,
            1 => ConditionCode::Ne,
            2 => ConditionCode::Cs,
            3 => ConditionCode::Cc,
            4 => ConditionCode::Mi,
            5 => ConditionCode::Pl,
            6 => ConditionCode::Vs,
            7 => ConditionCode::Vc,
            8 => ConditionCode::Hi,
            9 => ConditionCode::Ls,
            10 => ConditionCode::Ge,
            11 => ConditionCode::Lt,
            12 => ConditionCode::Gt,
            13 => ConditionCode::Le,
            14 => ConditionCode::Al,
            _ => ConditionCode::Nv,
        }
    }

    pub fn invert(self) -> Self {
        match self {
            ConditionCode::Eq => ConditionCode::Ne,
            ConditionCode::Ne => ConditionCode::Eq,
            ConditionCode::Cs => ConditionCode::Cc,
            ConditionCode::Cc => ConditionCode::Cs,
            ConditionCode::Mi => ConditionCode::Pl,
            ConditionCode::Pl => ConditionCode::Mi,
            ConditionCode::Vs => ConditionCode::Vc,
            ConditionCode::Vc => ConditionCode::Vs,
            ConditionCode::Hi => ConditionCode::Ls,
            ConditionCode::Ls => ConditionCode::Hi,
            ConditionCode::Ge => ConditionCode::Lt,
            ConditionCode::Lt => ConditionCode::Ge,
            ConditionCode::Gt => ConditionCode::Le,
            ConditionCode::Le => ConditionCode::Gt,
            ConditionCode::Al => ConditionCode::Nv,
            ConditionCode::Nv => ConditionCode::Al,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionCode::Eq => "eq",
            ConditionCode::Ne => "ne",
            ConditionCode::Cs => "cs",
            ConditionCode::Cc => "cc",
            ConditionCode::Mi => "mi",
            ConditionCode::Pl => "pl",
            ConditionCode::Vs => "vs",
            ConditionCode::Vc => "vc",
            ConditionCode::Hi => "hi",
            ConditionCode::Ls => "ls",
            ConditionCode::Ge => "ge",
            ConditionCode::Lt => "lt",
            ConditionCode::Gt => "gt",
            ConditionCode::Le => "le",
            ConditionCode::Al => "al",
            ConditionCode::Nv => "nv",
        }
    }
}

impl std::fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ExtendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtendType::Uxtb => "uxtb",
            ExtendType::Uxth => "uxth",
            ExtendType::Uxtw => "uxtw",
            ExtendType::Uxtx => "uxtx",
            ExtendType::Sxtb => "sxtb",
            ExtendType::Sxth => "sxth",
            ExtendType::Sxtw => "sxtw",
            ExtendType::Sxtx => "sxtx",
            ExtendType::Lsl => "lsl",
        }
    }
}
