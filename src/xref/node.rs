// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode {
    address: Address,
    name: Option<String>,
    kind: NodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Function,
    Data,
    String,
    Constant,
}

impl GraphNode {
    pub fn new(address: Address, kind: NodeKind) -> Self {
        Self {
            address,
            name: None,
            kind,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
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
        if let Some(name) = &self.name {
            write!(f, "{} @ {}", name, self.address)
        } else {
            write!(f, "{:?} @ {}", self.kind, self.address)
        }
    }
}
