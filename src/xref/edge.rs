// Tue Jan 15 2026 - Alex

use crate::memory::Address;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphEdge {
    from: Address,
    to: Address,
    kind: EdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Call,
    Data,
    String,
    Constant,
    Jump,
    Reference,
}

impl GraphEdge {
    pub fn new(from: Address, to: Address, kind: EdgeKind) -> Self {
        Self { from, to, kind }
    }

    pub fn from(&self) -> Address {
        self.from
    }

    pub fn to(&self) -> Address {
        self.to
    }

    pub fn kind(&self) -> EdgeKind {
        self.kind
    }

    pub fn is_call(&self) -> bool {
        matches!(self.kind, EdgeKind::Call)
    }
}

impl fmt::Display for GraphEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {} -> {}", self.kind, self.from, self.to)
    }
}
