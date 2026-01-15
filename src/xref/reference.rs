// Tue Jan 13 2026 - Alex

use crate::memory::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XRef {
    from: Address,
    to: Address,
    kind: XRefKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XRefKind {
    Call,
    Jump,
    Data,
    String,
}

impl XRef {
    pub fn new(from: Address, to: Address, kind: XRefKind) -> Self {
        Self { from, to, kind }
    }

    pub fn from(&self) -> Address {
        self.from
    }

    pub fn to(&self) -> Address {
        self.to
    }

    pub fn kind(&self) -> XRefKind {
        self.kind
    }

    pub fn is_call(&self) -> bool {
        matches!(self.kind, XRefKind::Call)
    }
}
