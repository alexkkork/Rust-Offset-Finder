// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::{Pattern, PatternMatcher};
use crate::symbol::SymbolResolver;
use crate::xref::XRefAnalyzer;
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct LuaCallFinder {
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: PatternMatcher,
    symbol_resolver: Option<Arc<SymbolResolver>>,
    xref_analyzer: Option<Arc<XRefAnalyzer>>,
}

impl LuaCallFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            pattern_matcher: PatternMatcher::new(reader),
            symbol_resolver: None,
            xref_analyzer: None,
        }
    }

    pub fn with_symbols(mut self, resolver: Arc<SymbolResolver>) -> Self {
        self.symbol_resolver = Some(resolver);
        self
    }

    pub fn with_xrefs(mut self, analyzer: Arc<XRefAnalyzer>) -> Self {
        self.xref_analyzer = Some(analyzer);
        self
    }

    pub fn find_lua_call(&self, start: Address, end: Address) -> Option<FinderResult> {
        if let Some(result) = self.find_by_symbol() {
            return Some(result);
        }

        if let Some(result) = self.find_by_pattern(start, end) {
            return Some(result);
        }

        if let Some(result) = self.find_by_xref(start, end) {
            return Some(result);
        }

        self.find_by_heuristic(start, end)
    }

    fn find_by_symbol(&self) -> Option<FinderResult> {
        let resolver = self.symbol_resolver.as_ref()?;

        let symbol_names = [
            "lua_call",
            "_lua_call",
            "luaD_call",
            "_luaD_call",
            "luaD_callnoyield",
        ];

        for name in &symbol_names {
            if let Some(addr) = resolver.resolve(name) {
                return Some(FinderResult {
                    name: "lua_call".to_string(),
                    address: addr,
                    confidence: 0.99,
                    method: "symbol".to_string(),
                    category: "lua_api".to_string(),
                    signature: Some("void lua_call(lua_State *L, int nargs, int nresults)".to_string()),
                });
            }
        }

        None
    }

    fn find_by_pattern(&self, start: Address, end: Address) -> Option<FinderResult> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? 39 ?? ?? ?? F9 ?? ?? ?? B4 ?? ?? ?? 94"),
            Pattern::from_hex("A9 ?? ?? ?? F9 ?? ?? ?? A9 ?? ?? ?? 94 ?? ?? ?? B4"),
            Pattern::from_hex("D1 ?? ?? ?? A9 ?? ?? ?? F9 ?? ?? ?? 91"),
        ];

        for pattern in patterns {
            let pattern_bytes = pattern.bytes();
            if let Ok(addrs) = self.pattern_matcher.find_pattern_in_range(
                pattern_bytes,
                start,
                end,
            ) {
                if let Some(addr) = addrs.first().copied() {
                    let func_start = self.find_function_start(addr);
                    if self.validate_lua_call(func_start) {
                        return Some(FinderResult {
                            name: "lua_call".to_string(),
                            address: func_start,
                            confidence: 0.85,
                            method: "pattern".to_string(),
                            category: "lua_api".to_string(),
                            signature: Some("void lua_call(lua_State *L, int nargs, int nresults)".to_string()),
                        });
                    }
                }
            }
        }

        None
    }

    fn find_by_xref(&self, start: Address, end: Address) -> Option<FinderResult> {
        let analyzer = self.xref_analyzer.as_ref()?;

        let error_strings = [
            "attempt to call",
            "stack overflow",
            "C stack overflow",
        ];

        for string in &error_strings {
            if let Some(string_addr) = self.find_string_reference(string, start, end) {
                let xrefs = analyzer.get_references_to(string_addr);

                for xref in xrefs {
                    let func_start = self.find_function_start(xref.from());
                    if self.validate_lua_call(func_start) {
                        return Some(FinderResult {
                            name: "lua_call".to_string(),
                            address: func_start,
                            confidence: 0.80,
                            method: "xref".to_string(),
                            category: "lua_api".to_string(),
                            signature: None,
                        });
                    }
                }
            }
        }

        None
    }

    fn find_by_heuristic(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 64) {
                if self.looks_like_lua_call(&bytes) {
                    let func_start = self.find_function_start(current);
                    if self.validate_lua_call(func_start) {
                        return Some(FinderResult {
                            name: "lua_call".to_string(),
                            address: func_start,
                            confidence: 0.70,
                            method: "heuristic".to_string(),
                            category: "lua_api".to_string(),
                            signature: None,
                        });
                    }
                }
            }

            current = current + 4;
        }

        None
    }

    fn find_function_start(&self, addr: Address) -> Address {
        let mut current = addr;
        let base = self.reader.get_base_address();

        while current > base {
            if let Ok(bytes) = self.reader.read_bytes(current, 4) {
                let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                if (insn & 0x7F800000) == 0x29000000 || (insn & 0x7F800000) == 0x6D000000 {
                    return current;
                }

                if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                    return current + 4;
                }
            }

            current = current - 4;
        }

        addr
    }

    fn validate_lua_call(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 128) {
            let mut has_state_access = false;
            let mut has_stack_check = false;
            let mut has_call_setup = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFFC00000) == 0xF9400000 {
                    has_state_access = true;
                }

                if (insn & 0x7F000000) == 0x71000000 || (insn & 0x7F000000) == 0x6B000000 {
                    has_stack_check = true;
                }

                if (insn & 0xFC000000) == 0x94000000 {
                    has_call_setup = true;
                }
            }

            return has_state_access && (has_stack_check || has_call_setup);
        }

        false
    }

    fn looks_like_lua_call(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 16 {
            return false;
        }

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return false;
        }

        let mut load_count = 0;
        let mut cmp_count = 0;

        for i in (0..bytes.len() - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFC00000) == 0xF9400000 {
                load_count += 1;
            }

            if (insn & 0x7F000000) == 0x71000000 {
                cmp_count += 1;
            }
        }

        load_count >= 2 && cmp_count >= 1
    }

    fn find_string_reference(&self, needle: &str, start: Address, end: Address) -> Option<Address> {
        let needle_bytes = needle.as_bytes();
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                if let Some(pos) = bytes.windows(needle_bytes.len())
                    .position(|w| w == needle_bytes)
                {
                    return Some(current + pos as u64);
                }
            }

            current = current + 4000;
        }

        None
    }
}

pub fn find_lua_call(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Option<FinderResult> {
    let finder = LuaCallFinder::new(reader);
    finder.find_lua_call(start, end)
}

pub type LuaApiCallAnalyzer = LuaCallFinder;
