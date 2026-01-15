// Tue Jan 15 2026 - Alex

use crate::memory::Address;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode {
    address: Address,
    name: String,
    kind: NodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Function,
    Data,
    String,
    Constant,
    External,
    Unknown,
}

impl GraphNode {
    pub fn new(address: Address, name: String, kind: NodeKind) -> Self {
        Self {
            address,
            name,
            kind,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, NodeKind::Function)
    }
}

impl fmt::Display for GraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.name, self.address)
    }
}
