// Tue Jan 13 2026 - Alex

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinderStrategy {
    Pattern,
    Symbol,
    XRef,
    Heuristic,
    Hybrid,
}

impl FinderStrategy {
    pub fn uses_pattern(&self) -> bool {
        matches!(self, Self::Pattern | Self::Hybrid)
    }

    pub fn uses_symbol(&self) -> bool {
        matches!(self, Self::Symbol | Self::Hybrid)
    }

    pub fn uses_xref(&self) -> bool {
        matches!(self, Self::XRef | Self::Hybrid)
    }

    pub fn uses_heuristic(&self) -> bool {
        matches!(self, Self::Heuristic | Self::Hybrid)
    }
}

impl Default for FinderStrategy {
    fn default() -> Self {
        Self::Hybrid
    }
}
