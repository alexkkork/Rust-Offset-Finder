// Tue Jan 13 2026 - Alex

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register {
    pub kind: RegisterKind,
    pub index: u8,
    pub is_64bit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegisterKind {
    General,
    FloatingPoint,
    Vector,
    System,
    Special,
}

impl Register {
    pub fn new(kind: RegisterKind, index: u8, is_64bit: bool) -> Self {
        Self { kind, index, is_64bit }
    }

    pub fn new_gpr(index: u8, is_64bit: bool) -> Self {
        Self::new(RegisterKind::General, index, is_64bit)
    }

    pub fn new_fp(index: u8) -> Self {
        Self::new(RegisterKind::FloatingPoint, index, true)
    }

    pub fn new_vec(index: u8) -> Self {
        Self::new(RegisterKind::Vector, index, true)
    }

    pub fn x(index: u8) -> Self {
        Self::new_gpr(index, true)
    }

    pub fn w(index: u8) -> Self {
        Self::new_gpr(index, false)
    }

    pub fn sp() -> Self {
        Self::new(RegisterKind::Special, 31, true)
    }

    pub fn xzr() -> Self {
        Self::new_gpr(31, true)
    }

    pub fn wzr() -> Self {
        Self::new_gpr(31, false)
    }

    pub fn lr() -> Self {
        Self::x(30)
    }

    pub fn fp() -> Self {
        Self::x(29)
    }

    pub fn pc() -> Self {
        Self::new(RegisterKind::Special, 32, true)
    }

    pub fn is_zero_register(&self) -> bool {
        self.kind == RegisterKind::General && self.index == 31
    }

    pub fn is_stack_pointer(&self) -> bool {
        self.kind == RegisterKind::Special && self.index == 31
    }

    pub fn is_link_register(&self) -> bool {
        self.kind == RegisterKind::General && self.index == 30
    }

    pub fn is_frame_pointer(&self) -> bool {
        self.kind == RegisterKind::General && self.index == 29
    }

    pub fn name(&self) -> String {
        match self.kind {
            RegisterKind::General => {
                if self.index == 31 {
                    if self.is_64bit { "xzr" } else { "wzr" }.to_string()
                } else {
                    let prefix = if self.is_64bit { "x" } else { "w" };
                    format!("{}{}", prefix, self.index)
                }
            }
            RegisterKind::FloatingPoint => {
                format!("d{}", self.index)
            }
            RegisterKind::Vector => {
                format!("v{}", self.index)
            }
            RegisterKind::System => {
                format!("sys{}", self.index)
            }
            RegisterKind::Special => {
                match self.index {
                    31 => "sp".to_string(),
                    32 => "pc".to_string(),
                    _ => format!("special{}", self.index),
                }
            }
        }
    }

    pub fn encoding(&self) -> u8 {
        self.index & 0x1F
    }

    pub fn size_bits(&self) -> usize {
        if self.is_64bit { 64 } else { 32 }
    }

    pub fn size_bytes(&self) -> usize {
        if self.is_64bit { 8 } else { 4 }
    }

    pub fn is_caller_saved(&self) -> bool {
        self.kind == RegisterKind::General && self.index <= 18
    }

    pub fn is_callee_saved(&self) -> bool {
        self.kind == RegisterKind::General && self.index >= 19 && self.index <= 28
    }

    pub fn is_argument(&self) -> bool {
        self.kind == RegisterKind::General && self.index <= 7
    }

    pub fn is_return_value(&self) -> bool {
        self.kind == RegisterKind::General && self.index == 0
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub const X0: Register = Register { kind: RegisterKind::General, index: 0, is_64bit: true };
pub const X1: Register = Register { kind: RegisterKind::General, index: 1, is_64bit: true };
pub const X2: Register = Register { kind: RegisterKind::General, index: 2, is_64bit: true };
pub const X3: Register = Register { kind: RegisterKind::General, index: 3, is_64bit: true };
pub const X4: Register = Register { kind: RegisterKind::General, index: 4, is_64bit: true };
pub const X5: Register = Register { kind: RegisterKind::General, index: 5, is_64bit: true };
pub const X6: Register = Register { kind: RegisterKind::General, index: 6, is_64bit: true };
pub const X7: Register = Register { kind: RegisterKind::General, index: 7, is_64bit: true };
pub const X8: Register = Register { kind: RegisterKind::General, index: 8, is_64bit: true };
pub const X9: Register = Register { kind: RegisterKind::General, index: 9, is_64bit: true };
pub const X10: Register = Register { kind: RegisterKind::General, index: 10, is_64bit: true };
pub const X11: Register = Register { kind: RegisterKind::General, index: 11, is_64bit: true };
pub const X12: Register = Register { kind: RegisterKind::General, index: 12, is_64bit: true };
pub const X13: Register = Register { kind: RegisterKind::General, index: 13, is_64bit: true };
pub const X14: Register = Register { kind: RegisterKind::General, index: 14, is_64bit: true };
pub const X15: Register = Register { kind: RegisterKind::General, index: 15, is_64bit: true };
pub const X16: Register = Register { kind: RegisterKind::General, index: 16, is_64bit: true };
pub const X17: Register = Register { kind: RegisterKind::General, index: 17, is_64bit: true };
pub const X18: Register = Register { kind: RegisterKind::General, index: 18, is_64bit: true };
pub const X19: Register = Register { kind: RegisterKind::General, index: 19, is_64bit: true };
pub const X20: Register = Register { kind: RegisterKind::General, index: 20, is_64bit: true };
pub const X21: Register = Register { kind: RegisterKind::General, index: 21, is_64bit: true };
pub const X22: Register = Register { kind: RegisterKind::General, index: 22, is_64bit: true };
pub const X23: Register = Register { kind: RegisterKind::General, index: 23, is_64bit: true };
pub const X24: Register = Register { kind: RegisterKind::General, index: 24, is_64bit: true };
pub const X25: Register = Register { kind: RegisterKind::General, index: 25, is_64bit: true };
pub const X26: Register = Register { kind: RegisterKind::General, index: 26, is_64bit: true };
pub const X27: Register = Register { kind: RegisterKind::General, index: 27, is_64bit: true };
pub const X28: Register = Register { kind: RegisterKind::General, index: 28, is_64bit: true };
pub const X29: Register = Register { kind: RegisterKind::General, index: 29, is_64bit: true };
pub const X30: Register = Register { kind: RegisterKind::General, index: 30, is_64bit: true };
pub const XZR: Register = Register { kind: RegisterKind::General, index: 31, is_64bit: true };

pub const W0: Register = Register { kind: RegisterKind::General, index: 0, is_64bit: false };
pub const W1: Register = Register { kind: RegisterKind::General, index: 1, is_64bit: false };
pub const W2: Register = Register { kind: RegisterKind::General, index: 2, is_64bit: false };
pub const W3: Register = Register { kind: RegisterKind::General, index: 3, is_64bit: false };
pub const W4: Register = Register { kind: RegisterKind::General, index: 4, is_64bit: false };
pub const W5: Register = Register { kind: RegisterKind::General, index: 5, is_64bit: false };
pub const W6: Register = Register { kind: RegisterKind::General, index: 6, is_64bit: false };
pub const W7: Register = Register { kind: RegisterKind::General, index: 7, is_64bit: false };
pub const WZR: Register = Register { kind: RegisterKind::General, index: 31, is_64bit: false };

pub const FP: Register = X29;
pub const LR: Register = X30;
pub const SP: Register = Register { kind: RegisterKind::Special, index: 31, is_64bit: true };
