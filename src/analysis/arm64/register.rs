// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register {
    pub bank: RegisterBank,
    pub index: u8,
    pub size: RegisterSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegisterBank {
    General,
    FloatingPoint,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegisterSize {
    Byte,
    Half,
    Word,
    Double,
    Quad,
}

impl Register {
    pub fn new(bank: RegisterBank, index: u8, size: RegisterSize) -> Self {
        Self { bank, index, size }
    }

    pub fn x(index: u8) -> Self {
        Self::new(RegisterBank::General, index, RegisterSize::Double)
    }

    pub fn w(index: u8) -> Self {
        Self::new(RegisterBank::General, index, RegisterSize::Word)
    }

    pub fn sp() -> Self {
        Self::new(RegisterBank::Special, 31, RegisterSize::Double)
    }

    pub fn xzr() -> Self {
        Self::new(RegisterBank::General, 31, RegisterSize::Double)
    }

    pub fn wzr() -> Self {
        Self::new(RegisterBank::General, 31, RegisterSize::Word)
    }

    pub fn pc() -> Self {
        Self::new(RegisterBank::Special, 32, RegisterSize::Double)
    }

    pub fn lr() -> Self {
        Self::x(30)
    }

    pub fn fp() -> Self {
        Self::x(29)
    }

    pub fn b(index: u8) -> Self {
        Self::new(RegisterBank::FloatingPoint, index, RegisterSize::Byte)
    }

    pub fn h(index: u8) -> Self {
        Self::new(RegisterBank::FloatingPoint, index, RegisterSize::Half)
    }

    pub fn s(index: u8) -> Self {
        Self::new(RegisterBank::FloatingPoint, index, RegisterSize::Word)
    }

    pub fn d(index: u8) -> Self {
        Self::new(RegisterBank::FloatingPoint, index, RegisterSize::Double)
    }

    pub fn q(index: u8) -> Self {
        Self::new(RegisterBank::FloatingPoint, index, RegisterSize::Quad)
    }

    pub fn v(index: u8, size: u8) -> Self {
        let reg_size = match size {
            0 => RegisterSize::Byte,
            1 => RegisterSize::Half,
            2 => RegisterSize::Word,
            3 => RegisterSize::Double,
            4 => RegisterSize::Quad,
            _ => RegisterSize::Quad,
        };
        Self::new(RegisterBank::FloatingPoint, index, reg_size)
    }

    pub fn bank(&self) -> RegisterBank {
        self.bank
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn size(&self) -> RegisterSize {
        self.size
    }

    pub fn is_general(&self) -> bool {
        self.bank == RegisterBank::General
    }

    pub fn is_floating_point(&self) -> bool {
        self.bank == RegisterBank::FloatingPoint
    }

    pub fn is_special(&self) -> bool {
        self.bank == RegisterBank::Special
    }

    pub fn is_zero_register(&self) -> bool {
        self.bank == RegisterBank::General && self.index == 31
    }

    pub fn is_stack_pointer(&self) -> bool {
        self.bank == RegisterBank::Special && self.index == 31
    }

    pub fn is_link_register(&self) -> bool {
        self.bank == RegisterBank::General && self.index == 30
    }

    pub fn is_frame_pointer(&self) -> bool {
        self.bank == RegisterBank::General && self.index == 29
    }

    pub fn is_callee_saved(&self) -> bool {
        if self.bank != RegisterBank::General {
            return false;
        }
        self.index >= 19 && self.index <= 28
    }

    pub fn is_caller_saved(&self) -> bool {
        if self.bank != RegisterBank::General {
            return false;
        }
        self.index <= 18
    }

    pub fn is_argument(&self) -> bool {
        if self.bank != RegisterBank::General {
            return false;
        }
        self.index <= 7
    }

    pub fn is_return_value(&self) -> bool {
        if self.bank != RegisterBank::General {
            return false;
        }
        self.index <= 1
    }

    pub fn overlaps(&self, other: &Register) -> bool {
        if self.bank != other.bank {
            return false;
        }
        self.index == other.index
    }

    pub fn to_64bit(&self) -> Self {
        if self.bank == RegisterBank::General {
            Self::x(self.index)
        } else {
            Self::new(self.bank, self.index, RegisterSize::Double)
        }
    }

    pub fn to_32bit(&self) -> Self {
        if self.bank == RegisterBank::General {
            Self::w(self.index)
        } else {
            Self::new(self.bank, self.index, RegisterSize::Word)
        }
    }
}

impl RegisterSize {
    pub fn bits(&self) -> u32 {
        match self {
            RegisterSize::Byte => 8,
            RegisterSize::Half => 16,
            RegisterSize::Word => 32,
            RegisterSize::Double => 64,
            RegisterSize::Quad => 128,
        }
    }

    pub fn bytes(&self) -> u8 {
        match self {
            RegisterSize::Byte => 1,
            RegisterSize::Half => 2,
            RegisterSize::Word => 4,
            RegisterSize::Double => 8,
            RegisterSize::Quad => 16,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.bank {
            RegisterBank::General => {
                if self.index == 31 {
                    match self.size {
                        RegisterSize::Word => write!(f, "wzr"),
                        _ => write!(f, "xzr"),
                    }
                } else {
                    match self.size {
                        RegisterSize::Word => write!(f, "w{}", self.index),
                        _ => write!(f, "x{}", self.index),
                    }
                }
            }
            RegisterBank::FloatingPoint => {
                let prefix = match self.size {
                    RegisterSize::Byte => 'b',
                    RegisterSize::Half => 'h',
                    RegisterSize::Word => 's',
                    RegisterSize::Double => 'd',
                    RegisterSize::Quad => 'q',
                };
                write!(f, "{}{}", prefix, self.index)
            }
            RegisterBank::Special => {
                match self.index {
                    31 => write!(f, "sp"),
                    32 => write!(f, "pc"),
                    _ => write!(f, "sr{}", self.index),
                }
            }
        }
    }
}

pub fn parse_register(s: &str) -> Option<Register> {
    let s = s.to_lowercase();
    if s == "sp" {
        return Some(Register::sp());
    }
    if s == "pc" {
        return Some(Register::pc());
    }
    if s == "lr" {
        return Some(Register::lr());
    }
    if s == "fp" {
        return Some(Register::fp());
    }
    if s == "xzr" {
        return Some(Register::xzr());
    }
    if s == "wzr" {
        return Some(Register::wzr());
    }

    let first = s.chars().next()?;
    let index: u8 = s[1..].parse().ok()?;

    match first {
        'x' => Some(Register::x(index)),
        'w' => Some(Register::w(index)),
        'b' => Some(Register::b(index)),
        'h' => Some(Register::h(index)),
        's' => Some(Register::s(index)),
        'd' => Some(Register::d(index)),
        'q' => Some(Register::q(index)),
        'v' => Some(Register::v(index, 4)),
        _ => None,
    }
}

pub const X0: Register = Register { bank: RegisterBank::General, index: 0, size: RegisterSize::Double };
pub const X1: Register = Register { bank: RegisterBank::General, index: 1, size: RegisterSize::Double };
pub const X2: Register = Register { bank: RegisterBank::General, index: 2, size: RegisterSize::Double };
pub const X3: Register = Register { bank: RegisterBank::General, index: 3, size: RegisterSize::Double };
pub const X4: Register = Register { bank: RegisterBank::General, index: 4, size: RegisterSize::Double };
pub const X5: Register = Register { bank: RegisterBank::General, index: 5, size: RegisterSize::Double };
pub const X6: Register = Register { bank: RegisterBank::General, index: 6, size: RegisterSize::Double };
pub const X7: Register = Register { bank: RegisterBank::General, index: 7, size: RegisterSize::Double };
pub const X8: Register = Register { bank: RegisterBank::General, index: 8, size: RegisterSize::Double };
pub const X9: Register = Register { bank: RegisterBank::General, index: 9, size: RegisterSize::Double };
pub const X10: Register = Register { bank: RegisterBank::General, index: 10, size: RegisterSize::Double };
pub const X11: Register = Register { bank: RegisterBank::General, index: 11, size: RegisterSize::Double };
pub const X12: Register = Register { bank: RegisterBank::General, index: 12, size: RegisterSize::Double };
pub const X13: Register = Register { bank: RegisterBank::General, index: 13, size: RegisterSize::Double };
pub const X14: Register = Register { bank: RegisterBank::General, index: 14, size: RegisterSize::Double };
pub const X15: Register = Register { bank: RegisterBank::General, index: 15, size: RegisterSize::Double };
pub const X16: Register = Register { bank: RegisterBank::General, index: 16, size: RegisterSize::Double };
pub const X17: Register = Register { bank: RegisterBank::General, index: 17, size: RegisterSize::Double };
pub const X18: Register = Register { bank: RegisterBank::General, index: 18, size: RegisterSize::Double };
pub const X19: Register = Register { bank: RegisterBank::General, index: 19, size: RegisterSize::Double };
pub const X20: Register = Register { bank: RegisterBank::General, index: 20, size: RegisterSize::Double };
pub const X21: Register = Register { bank: RegisterBank::General, index: 21, size: RegisterSize::Double };
pub const X22: Register = Register { bank: RegisterBank::General, index: 22, size: RegisterSize::Double };
pub const X23: Register = Register { bank: RegisterBank::General, index: 23, size: RegisterSize::Double };
pub const X24: Register = Register { bank: RegisterBank::General, index: 24, size: RegisterSize::Double };
pub const X25: Register = Register { bank: RegisterBank::General, index: 25, size: RegisterSize::Double };
pub const X26: Register = Register { bank: RegisterBank::General, index: 26, size: RegisterSize::Double };
pub const X27: Register = Register { bank: RegisterBank::General, index: 27, size: RegisterSize::Double };
pub const X28: Register = Register { bank: RegisterBank::General, index: 28, size: RegisterSize::Double };
pub const X29: Register = Register { bank: RegisterBank::General, index: 29, size: RegisterSize::Double };
pub const X30: Register = Register { bank: RegisterBank::General, index: 30, size: RegisterSize::Double };
pub const XZR: Register = Register { bank: RegisterBank::General, index: 31, size: RegisterSize::Double };
pub const SP: Register = Register { bank: RegisterBank::Special, index: 31, size: RegisterSize::Double };
pub const PC: Register = Register { bank: RegisterBank::Special, index: 32, size: RegisterSize::Double };
pub const LR: Register = X30;
pub const FP: Register = X29;
