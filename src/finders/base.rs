// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::{FinderResult, FinderError, FinderStrategy};
use std::sync::Arc;

pub trait OffsetFinder: Send + Sync {
    fn find(&self, strategy: FinderStrategy) -> Result<Option<FinderResult>, FinderError>;
    fn find_all(&self, strategy: FinderStrategy) -> Result<Vec<FinderResult>, FinderError>;
    fn name(&self) -> &str;
}

pub struct BaseFinder {
    name: String,
    strategy: FinderStrategy,
}

impl BaseFinder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            strategy: FinderStrategy::Hybrid,
        }
    }

    pub fn with_strategy(mut self, strategy: FinderStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn strategy(&self) -> FinderStrategy {
        self.strategy
    }
}
