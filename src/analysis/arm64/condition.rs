// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Condition {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
    NV,
}

impl Condition {
    pub fn from_code(code: u8) -> Self {
        match code & 0xF {
            0x0 => Condition::EQ,
            0x1 => Condition::NE,
            0x2 => Condition::CS,
            0x3 => Condition::CC,
            0x4 => Condition::MI,
            0x5 => Condition::PL,
            0x6 => Condition::VS,
            0x7 => Condition::VC,
            0x8 => Condition::HI,
            0x9 => Condition::LS,
            0xA => Condition::GE,
            0xB => Condition::LT,
            0xC => Condition::GT,
            0xD => Condition::LE,
            0xE => Condition::AL,
            0xF => Condition::NV,
            _ => Condition::AL,
        }
    }

    pub fn to_code(self) -> u8 {
        match self {
            Condition::EQ => 0x0,
            Condition::NE => 0x1,
            Condition::CS => 0x2,
            Condition::CC => 0x3,
            Condition::MI => 0x4,
            Condition::PL => 0x5,
            Condition::VS => 0x6,
            Condition::VC => 0x7,
            Condition::HI => 0x8,
            Condition::LS => 0x9,
            Condition::GE => 0xA,
            Condition::LT => 0xB,
            Condition::GT => 0xC,
            Condition::LE => 0xD,
            Condition::AL => 0xE,
            Condition::NV => 0xF,
        }
    }

    pub fn invert(self) -> Self {
        match self {
            Condition::EQ => Condition::NE,
            Condition::NE => Condition::EQ,
            Condition::CS => Condition::CC,
            Condition::CC => Condition::CS,
            Condition::MI => Condition::PL,
            Condition::PL => Condition::MI,
            Condition::VS => Condition::VC,
            Condition::VC => Condition::VS,
            Condition::HI => Condition::LS,
            Condition::LS => Condition::HI,
            Condition::GE => Condition::LT,
            Condition::LT => Condition::GE,
            Condition::GT => Condition::LE,
            Condition::LE => Condition::GT,
            Condition::AL => Condition::NV,
            Condition::NV => Condition::AL,
        }
    }

    pub fn mnemonic(self) -> &'static str {
        match self {
            Condition::EQ => "eq",
            Condition::NE => "ne",
            Condition::CS => "cs",
            Condition::CC => "cc",
            Condition::MI => "mi",
            Condition::PL => "pl",
            Condition::VS => "vs",
            Condition::VC => "vc",
            Condition::HI => "hi",
            Condition::LS => "ls",
            Condition::GE => "ge",
            Condition::LT => "lt",
            Condition::GT => "gt",
            Condition::LE => "le",
            Condition::AL => "al",
            Condition::NV => "nv",
        }
    }

    pub fn alternate_mnemonic(self) -> &'static str {
        match self {
            Condition::CS => "hs",
            Condition::CC => "lo",
            _ => self.mnemonic(),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Condition::EQ => "Equal (Z == 1)",
            Condition::NE => "Not equal (Z == 0)",
            Condition::CS => "Carry set / unsigned higher or same (C == 1)",
            Condition::CC => "Carry clear / unsigned lower (C == 0)",
            Condition::MI => "Minus / negative (N == 1)",
            Condition::PL => "Plus / positive or zero (N == 0)",
            Condition::VS => "Overflow (V == 1)",
            Condition::VC => "No overflow (V == 0)",
            Condition::HI => "Unsigned higher (C == 1 && Z == 0)",
            Condition::LS => "Unsigned lower or same (C == 0 || Z == 1)",
            Condition::GE => "Signed greater than or equal (N == V)",
            Condition::LT => "Signed less than (N != V)",
            Condition::GT => "Signed greater than (Z == 0 && N == V)",
            Condition::LE => "Signed less than or equal (Z == 1 || N != V)",
            Condition::AL => "Always (unconditional)",
            Condition::NV => "Never (reserved)",
        }
    }

    pub fn is_always(self) -> bool {
        matches!(self, Condition::AL | Condition::NV)
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Condition::GE | Condition::LT | Condition::GT | Condition::LE | Condition::MI | Condition::PL | Condition::VS | Condition::VC)
    }

    pub fn is_unsigned(self) -> bool {
        matches!(self, Condition::CS | Condition::CC | Condition::HI | Condition::LS)
    }

    pub fn evaluate(&self, n: bool, z: bool, c: bool, v: bool) -> bool {
        match self {
            Condition::EQ => z,
            Condition::NE => !z,
            Condition::CS => c,
            Condition::CC => !c,
            Condition::MI => n,
            Condition::PL => !n,
            Condition::VS => v,
            Condition::VC => !v,
            Condition::HI => c && !z,
            Condition::LS => !c || z,
            Condition::GE => n == v,
            Condition::LT => n != v,
            Condition::GT => !z && (n == v),
            Condition::LE => z || (n != v),
            Condition::AL => true,
            Condition::NV => false,
        }
    }

    pub fn required_flags(self) -> (bool, bool, bool, bool) {
        match self {
            Condition::EQ | Condition::NE => (false, true, false, false),
            Condition::CS | Condition::CC => (false, false, true, false),
            Condition::MI | Condition::PL => (true, false, false, false),
            Condition::VS | Condition::VC => (false, false, false, true),
            Condition::HI | Condition::LS => (false, true, true, false),
            Condition::GE | Condition::LT => (true, false, false, true),
            Condition::GT | Condition::LE => (true, true, false, true),
            Condition::AL | Condition::NV => (false, false, false, false),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mnemonic())
    }
}

pub fn parse_condition(s: &str) -> Option<Condition> {
    match s.to_lowercase().as_str() {
        "eq" => Some(Condition::EQ),
        "ne" => Some(Condition::NE),
        "cs" | "hs" => Some(Condition::CS),
        "cc" | "lo" => Some(Condition::CC),
        "mi" => Some(Condition::MI),
        "pl" => Some(Condition::PL),
        "vs" => Some(Condition::VS),
        "vc" => Some(Condition::VC),
        "hi" => Some(Condition::HI),
        "ls" => Some(Condition::LS),
        "ge" => Some(Condition::GE),
        "lt" => Some(Condition::LT),
        "gt" => Some(Condition::GT),
        "le" => Some(Condition::LE),
        "al" => Some(Condition::AL),
        "nv" => Some(Condition::NV),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub n: bool,
    pub z: bool,
    pub c: bool,
    pub v: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bits(nzcv: u8) -> Self {
        Self {
            n: (nzcv & 0x8) != 0,
            z: (nzcv & 0x4) != 0,
            c: (nzcv & 0x2) != 0,
            v: (nzcv & 0x1) != 0,
        }
    }

    pub fn to_bits(&self) -> u8 {
        let mut bits = 0u8;
        if self.n { bits |= 0x8; }
        if self.z { bits |= 0x4; }
        if self.c { bits |= 0x2; }
        if self.v { bits |= 0x1; }
        bits
    }

    pub fn evaluate(&self, condition: Condition) -> bool {
        condition.evaluate(self.n, self.z, self.c, self.v)
    }

    pub fn set_add(&mut self, op1: u64, op2: u64, result: u64, is_64bit: bool) {
        let msb = if is_64bit { 63 } else { 31 };
        self.n = (result >> msb) & 1 != 0;
        self.z = result == 0;
        self.c = result < op1;
        let sign1 = (op1 >> msb) & 1;
        let sign2 = (op2 >> msb) & 1;
        let sign_r = (result >> msb) & 1;
        self.v = (sign1 == sign2) && (sign1 != sign_r);
    }

    pub fn set_sub(&mut self, op1: u64, op2: u64, result: u64, is_64bit: bool) {
        let msb = if is_64bit { 63 } else { 31 };
        self.n = (result >> msb) & 1 != 0;
        self.z = result == 0;
        self.c = op1 >= op2;
        let sign1 = (op1 >> msb) & 1;
        let sign2 = (op2 >> msb) & 1;
        let sign_r = (result >> msb) & 1;
        self.v = (sign1 != sign2) && (sign1 != sign_r);
    }

    pub fn set_logical(&mut self, result: u64, is_64bit: bool) {
        let msb = if is_64bit { 63 } else { 31 };
        self.n = (result >> msb) & 1 != 0;
        self.z = result == 0;
        self.c = false;
        self.v = false;
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            if self.n { 'N' } else { 'n' },
            if self.z { 'Z' } else { 'z' },
            if self.c { 'C' } else { 'c' },
            if self.v { 'V' } else { 'v' },
        )
    }
}
