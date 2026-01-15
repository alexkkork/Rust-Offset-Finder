// Tue Jan 13 2026 - Alex

use crate::finders::{OffsetFinder, FinderResult, FinderError, FinderStrategy};

pub struct LuaTypeFinder {
    finder: crate::finders::lua_api::finder::LuaApiFinder,
}

impl LuaTypeFinder {
    pub fn new() -> Self {
        Self {
            finder: crate::finders::lua_api::finder::LuaApiFinder::new(
                "lua_type".to_string(),
                vec![0x00, 0x00, 0x00, 0x00],
                "lua_type".to_string(),
            ),
        }
    }
}

impl OffsetFinder for LuaTypeFinder {
    fn find(&self, strategy: FinderStrategy) -> Result<Option<FinderResult>, FinderError> {
        self.finder.find(strategy)
    }

    fn find_all(&self, strategy: FinderStrategy) -> Result<Vec<FinderResult>, FinderError> {
        self.finder.find_all(strategy)
    }

    fn name(&self) -> &str {
        self.finder.name()
    }
}

impl Default for LuaTypeFinder {
    fn default() -> Self {
        Self::new()
    }
}
