// Tue Jan 13 2026 - Alex

use crate::finders::{OffsetFinder, FinderResult, FinderError, FinderStrategy};

pub struct LuaRawSetIFinder {
    finder: crate::finders::lua_api::finder::LuaApiFinder,
}

impl LuaRawSetIFinder {
    pub fn new() -> Self {
        Self {
            finder: crate::finders::lua_api::finder::LuaApiFinder::new(
                "lua_rawseti".to_string(),
                vec![0x00, 0x00, 0x00, 0x00],
                "lua_rawseti".to_string(),
            ),
        }
    }
}

impl OffsetFinder for LuaRawSetIFinder {
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

impl Default for LuaRawSetIFinder {
    fn default() -> Self {
        Self::new()
    }
}
