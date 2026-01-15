// Wed Jan 15 2026 - Alex

pub mod decoder;
pub mod encoder;
pub mod patterns;

pub use decoder::Arm64Decoder;
pub use encoder::Arm64Encoder;
pub use patterns::Arm64Patterns;

use crate::memory::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm64Register {
    X(u8),
    W(u8),
    Sp,
    Xzr,
    Wzr,
    Pc,
    V(u8),
    B(u8),
    H(u8),
    S(u8),
    D(u8),
    Q(u8),
}

impl Arm64Register {
    pub fn from_encoding(reg: u8, is_64bit: bool) -> Self {
        if reg == 31 {
            if is_64bit { Arm64Register::Xzr } else { Arm64Register::Wzr }
        } else if is_64bit {
            Arm64Register::X(reg)
        } else {
            Arm64Register::W(reg)
        }
    }

    pub fn name(&self) -> String {
        match self {
            Arm64Register::X(n) => format!("X{}", n),
            Arm64Register::W(n) => format!("W{}", n),
            Arm64Register::Sp => "SP".to_string(),
            Arm64Register::Xzr => "XZR".to_string(),
            Arm64Register::Wzr => "WZR".to_string(),
            Arm64Register::Pc => "PC".to_string(),
            Arm64Register::V(n) => format!("V{}", n),
            Arm64Register::B(n) => format!("B{}", n),
            Arm64Register::H(n) => format!("H{}", n),
            Arm64Register::S(n) => format!("S{}", n),
            Arm64Register::D(n) => format!("D{}", n),
            Arm64Register::Q(n) => format!("Q{}", n),
        }
    }

    pub fn is_64bit(&self) -> bool {
        matches!(self, Arm64Register::X(_) | Arm64Register::Sp | Arm64Register::Xzr | Arm64Register::Pc)
    }

    pub fn encoding(&self) -> u8 {
        match self {
            Arm64Register::X(n) | Arm64Register::W(n) |
            Arm64Register::V(n) | Arm64Register::B(n) |
            Arm64Register::H(n) | Arm64Register::S(n) |
            Arm64Register::D(n) | Arm64Register::Q(n) => *n,
            Arm64Register::Sp => 31,
            Arm64Register::Xzr | Arm64Register::Wzr => 31,
            Arm64Register::Pc => 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm64Condition {
    Eq,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
    Nv,
}

impl Arm64Condition {
    pub fn from_encoding(cond: u8) -> Self {
        match cond & 0xF {
            0x0 => Arm64Condition::Eq,
            0x1 => Arm64Condition::Ne,
            0x2 => Arm64Condition::Cs,
            0x3 => Arm64Condition::Cc,
            0x4 => Arm64Condition::Mi,
            0x5 => Arm64Condition::Pl,
            0x6 => Arm64Condition::Vs,
            0x7 => Arm64Condition::Vc,
            0x8 => Arm64Condition::Hi,
            0x9 => Arm64Condition::Ls,
            0xA => Arm64Condition::Ge,
            0xB => Arm64Condition::Lt,
            0xC => Arm64Condition::Gt,
            0xD => Arm64Condition::Le,
            0xE => Arm64Condition::Al,
            0xF => Arm64Condition::Nv,
            _ => unreachable!(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Arm64Condition::Eq => "EQ",
            Arm64Condition::Ne => "NE",
            Arm64Condition::Cs => "CS",
            Arm64Condition::Cc => "CC",
            Arm64Condition::Mi => "MI",
            Arm64Condition::Pl => "PL",
            Arm64Condition::Vs => "VS",
            Arm64Condition::Vc => "VC",
            Arm64Condition::Hi => "HI",
            Arm64Condition::Ls => "LS",
            Arm64Condition::Ge => "GE",
            Arm64Condition::Lt => "LT",
            Arm64Condition::Gt => "GT",
            Arm64Condition::Le => "LE",
            Arm64Condition::Al => "AL",
            Arm64Condition::Nv => "NV",
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            Arm64Condition::Eq => Arm64Condition::Ne,
            Arm64Condition::Ne => Arm64Condition::Eq,
            Arm64Condition::Cs => Arm64Condition::Cc,
            Arm64Condition::Cc => Arm64Condition::Cs,
            Arm64Condition::Mi => Arm64Condition::Pl,
            Arm64Condition::Pl => Arm64Condition::Mi,
            Arm64Condition::Vs => Arm64Condition::Vc,
            Arm64Condition::Vc => Arm64Condition::Vs,
            Arm64Condition::Hi => Arm64Condition::Ls,
            Arm64Condition::Ls => Arm64Condition::Hi,
            Arm64Condition::Ge => Arm64Condition::Lt,
            Arm64Condition::Lt => Arm64Condition::Ge,
            Arm64Condition::Gt => Arm64Condition::Le,
            Arm64Condition::Le => Arm64Condition::Gt,
            Arm64Condition::Al => Arm64Condition::Nv,
            Arm64Condition::Nv => Arm64Condition::Al,
        }
    }

    pub fn encoding(&self) -> u8 {
        match self {
            Arm64Condition::Eq => 0x0,
            Arm64Condition::Ne => 0x1,
            Arm64Condition::Cs => 0x2,
            Arm64Condition::Cc => 0x3,
            Arm64Condition::Mi => 0x4,
            Arm64Condition::Pl => 0x5,
            Arm64Condition::Vs => 0x6,
            Arm64Condition::Vc => 0x7,
            Arm64Condition::Hi => 0x8,
            Arm64Condition::Ls => 0x9,
            Arm64Condition::Ge => 0xA,
            Arm64Condition::Lt => 0xB,
            Arm64Condition::Gt => 0xC,
            Arm64Condition::Le => 0xD,
            Arm64Condition::Al => 0xE,
            Arm64Condition::Nv => 0xF,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arm64Instruction {
    pub address: Address,
    pub raw: u32,
    pub mnemonic: String,
    pub operands: Vec<Arm64Operand>,
    pub writes_flags: bool,
    pub reads_flags: bool,
}

#[derive(Debug, Clone)]
pub enum Arm64Operand {
    Register(Arm64Register),
    Immediate(i64),
    Address(Address),
    Memory { base: Arm64Register, offset: i64, pre_index: bool, post_index: bool },
    ShiftedReg { reg: Arm64Register, shift: Arm64Shift, amount: u8 },
    ExtendedReg { reg: Arm64Register, extend: Arm64Extend, shift: u8 },
    Condition(Arm64Condition),
    Label(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm64Shift {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

impl Arm64Shift {
    pub fn from_encoding(shift: u8) -> Self {
        match shift & 0x3 {
            0 => Arm64Shift::Lsl,
            1 => Arm64Shift::Lsr,
            2 => Arm64Shift::Asr,
            3 => Arm64Shift::Ror,
            _ => unreachable!(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Arm64Shift::Lsl => "LSL",
            Arm64Shift::Lsr => "LSR",
            Arm64Shift::Asr => "ASR",
            Arm64Shift::Ror => "ROR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm64Extend {
    Uxtb,
    Uxth,
    Uxtw,
    Uxtx,
    Sxtb,
    Sxth,
    Sxtw,
    Sxtx,
}

impl Arm64Extend {
    pub fn from_encoding(extend: u8) -> Self {
        match extend & 0x7 {
            0 => Arm64Extend::Uxtb,
            1 => Arm64Extend::Uxth,
            2 => Arm64Extend::Uxtw,
            3 => Arm64Extend::Uxtx,
            4 => Arm64Extend::Sxtb,
            5 => Arm64Extend::Sxth,
            6 => Arm64Extend::Sxtw,
            7 => Arm64Extend::Sxtx,
            _ => unreachable!(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Arm64Extend::Uxtb => "UXTB",
            Arm64Extend::Uxth => "UXTH",
            Arm64Extend::Uxtw => "UXTW",
            Arm64Extend::Uxtx => "UXTX",
            Arm64Extend::Sxtb => "SXTB",
            Arm64Extend::Sxth => "SXTH",
            Arm64Extend::Sxtw => "SXTW",
            Arm64Extend::Sxtx => "SXTX",
        }
    }
}
