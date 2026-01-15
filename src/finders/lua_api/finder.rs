// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::{Pattern, PatternMatcher};
use crate::symbol::SymbolResolver;
use crate::xref::XRefAnalyzer;
use crate::finders::result::FinderResult;
use std::sync::Arc;
use std::collections::HashMap;

pub struct LuaApiFinder {
    reader: Arc<dyn MemoryReader>,
    pattern_matcher: PatternMatcher,
    symbol_resolver: Option<Arc<SymbolResolver>>,
    xref_analyzer: Option<Arc<XRefAnalyzer>>,
    found_functions: HashMap<String, FinderResult>,
}

impl LuaApiFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            pattern_matcher: PatternMatcher::new(),
            symbol_resolver: None,
            xref_analyzer: None,
            found_functions: HashMap::new(),
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

    pub fn find_all(&mut self, start: Address, end: Address) -> Vec<FinderResult> {
        let mut results = Vec::new();

        let functions = self.get_target_functions();

        for (name, patterns, symbol_names) in functions {
            if let Some(result) = self.find_function(&name, &patterns, &symbol_names, start, end) {
                self.found_functions.insert(name, result.clone());
                results.push(result);
            }
        }

        self.cross_reference_validate(&mut results);

        results
    }

    fn get_target_functions(&self) -> Vec<(String, Vec<&'static str>, Vec<&'static str>)> {
        vec![
            (
                "lua_gettop".to_string(),
                vec!["F9 ?? ?? ?? D1 ?? ?? ?? 9B ?? ?? ?? CB"],
                vec!["lua_gettop", "_lua_gettop"],
            ),
            (
                "lua_settop".to_string(),
                vec!["F9 ?? ?? ?? B4 ?? ?? ?? 91 ?? ?? ?? F9"],
                vec!["lua_settop", "_lua_settop"],
            ),
            (
                "lua_pushvalue".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 91 ?? ?? ?? EB"],
                vec!["lua_pushvalue", "_lua_pushvalue"],
            ),
            (
                "lua_type".to_string(),
                vec!["F9 ?? ?? ?? B4 ?? ?? ?? 39 ?? ?? ?? 52"],
                vec!["lua_type", "_lua_type"],
            ),
            (
                "lua_tonumber".to_string(),
                vec!["F9 ?? ?? ?? BD ?? ?? ?? 1E ?? ?? ?? 1E"],
                vec!["lua_tonumber", "_lua_tonumber", "lua_tonumberx"],
            ),
            (
                "lua_toboolean".to_string(),
                vec!["F9 ?? ?? ?? B4 ?? ?? ?? 39 ?? ?? ?? 71"],
                vec!["lua_toboolean", "_lua_toboolean"],
            ),
            (
                "lua_tostring".to_string(),
                vec!["F9 ?? ?? ?? B4 ?? ?? ?? 39 ?? ?? ?? F0"],
                vec!["lua_tostring", "_lua_tostring", "lua_tolstring"],
            ),
            (
                "lua_touserdata".to_string(),
                vec!["F9 ?? ?? ?? B4 ?? ?? ?? 39 ?? ?? ?? 71 ?? ?? ?? 54"],
                vec!["lua_touserdata", "_lua_touserdata"],
            ),
            (
                "lua_rawget".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 94 ?? ?? ?? A9"],
                vec!["lua_rawget", "_lua_rawget"],
            ),
            (
                "lua_rawgeti".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 93 ?? ?? ?? 94"],
                vec!["lua_rawgeti", "_lua_rawgeti"],
            ),
            (
                "lua_rawset".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? D1 ?? ?? ?? F9"],
                vec!["lua_rawset", "_lua_rawset"],
            ),
            (
                "lua_rawseti".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 93 ?? ?? ?? D1"],
                vec!["lua_rawseti", "_lua_rawseti"],
            ),
            (
                "lua_getfield".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 94 ?? ?? ?? B4"],
                vec!["lua_getfield", "_lua_getfield"],
            ),
            (
                "lua_createtable".to_string(),
                vec!["A9 ?? ?? ?? 2A ?? ?? ?? 2A ?? ?? ?? 94"],
                vec!["lua_createtable", "_lua_createtable"],
            ),
            (
                "lua_newthread".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? 52 ?? ?? ?? 94"],
                vec!["lua_newthread", "_lua_newthread"],
            ),
            (
                "lua_resume".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? B4 ?? ?? ?? F9 ?? ?? ?? 94"],
                vec!["lua_resume", "_lua_resume"],
            ),
            (
                "lua_pcall".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? D1 ?? ?? ?? 94 ?? ?? ?? B5"],
                vec!["lua_pcall", "_lua_pcall", "lua_pcallk"],
            ),
            (
                "lua_call".to_string(),
                vec!["A9 ?? ?? ?? F9 ?? ?? ?? D1 ?? ?? ?? 94 ?? ?? ?? A9"],
                vec!["lua_call", "_lua_call", "lua_callk"],
            ),
        ]
    }

    fn find_function(
        &self,
        name: &str,
        patterns: &[&str],
        symbol_names: &[&str],
        start: Address,
        end: Address,
    ) -> Option<FinderResult> {
        if let Some(result) = self.find_by_symbol(name, symbol_names) {
            return Some(result);
        }

        if let Some(result) = self.find_by_patterns(name, patterns, start, end) {
            return Some(result);
        }

        self.find_by_xref_heuristic(name, start, end)
    }

    fn find_by_symbol(&self, name: &str, symbol_names: &[&str]) -> Option<FinderResult> {
        let resolver = self.symbol_resolver.as_ref()?;

        for symbol_name in symbol_names {
            if let Some(addr) = resolver.resolve(symbol_name) {
                return Some(FinderResult {
                    name: name.to_string(),
                    address: addr,
                    confidence: 0.99,
                    method: "symbol".to_string(),
                    category: "lua_api".to_string(),
                    signature: self.get_signature(name),
                });
            }
        }

        None
    }

    fn find_by_patterns(
        &self,
        name: &str,
        patterns: &[&str],
        start: Address,
        end: Address,
    ) -> Option<FinderResult> {
        for pattern_str in patterns {
            let pattern = Pattern::from_hex(pattern_str);

            if let Some(addr) = self.pattern_matcher.find_pattern_in_range(
                self.reader.as_ref(),
                &pattern,
                start,
                end,
            ) {
                let func_start = self.find_function_start(addr);

                if self.validate_lua_function(name, func_start) {
                    return Some(FinderResult {
                        name: name.to_string(),
                        address: func_start,
                        confidence: 0.85,
                        method: "pattern".to_string(),
                        category: "lua_api".to_string(),
                        signature: self.get_signature(name),
                    });
                }
            }
        }

        None
    }

    fn find_by_xref_heuristic(
        &self,
        name: &str,
        start: Address,
        end: Address,
    ) -> Option<FinderResult> {
        if let Some(gettop) = self.found_functions.get("lua_gettop") {
            if let Some(analyzer) = &self.xref_analyzer {
                let callers = analyzer.get_references_to(gettop.address);

                for caller in callers.iter().take(50) {
                    let func_start = self.find_function_start(*caller);

                    if self.validate_lua_function(name, func_start) {
                        return Some(FinderResult {
                            name: name.to_string(),
                            address: func_start,
                            confidence: 0.70,
                            method: "xref_heuristic".to_string(),
                            category: "lua_api".to_string(),
                            signature: self.get_signature(name),
                        });
                    }
                }
            }
        }

        None
    }

    fn find_function_start(&self, addr: Address) -> Address {
        let mut current = addr;
        let base = self.reader.get_base_address();

        for _ in 0..256 {
            if current <= base {
                break;
            }

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

    fn validate_lua_function(&self, name: &str, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 64) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
                return false;
            }

            let mut has_state_access = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFFC00000) == 0xF9400000 {
                    has_state_access = true;
                    break;
                }
            }

            return has_state_access;
        }

        false
    }

    fn get_signature(&self, name: &str) -> Option<String> {
        match name {
            "lua_gettop" => Some("int lua_gettop(lua_State *L)".to_string()),
            "lua_settop" => Some("void lua_settop(lua_State *L, int index)".to_string()),
            "lua_pushvalue" => Some("void lua_pushvalue(lua_State *L, int index)".to_string()),
            "lua_type" => Some("int lua_type(lua_State *L, int index)".to_string()),
            "lua_tonumber" => Some("lua_Number lua_tonumber(lua_State *L, int index)".to_string()),
            "lua_toboolean" => Some("int lua_toboolean(lua_State *L, int index)".to_string()),
            "lua_tostring" => Some("const char* lua_tostring(lua_State *L, int index)".to_string()),
            "lua_touserdata" => Some("void* lua_touserdata(lua_State *L, int index)".to_string()),
            "lua_rawget" => Some("void lua_rawget(lua_State *L, int index)".to_string()),
            "lua_rawgeti" => Some("void lua_rawgeti(lua_State *L, int index, int n)".to_string()),
            "lua_rawset" => Some("void lua_rawset(lua_State *L, int index)".to_string()),
            "lua_rawseti" => Some("void lua_rawseti(lua_State *L, int index, int n)".to_string()),
            "lua_getfield" => Some("void lua_getfield(lua_State *L, int index, const char *k)".to_string()),
            "lua_createtable" => Some("void lua_createtable(lua_State *L, int narr, int nrec)".to_string()),
            "lua_newthread" => Some("lua_State* lua_newthread(lua_State *L)".to_string()),
            "lua_resume" => Some("int lua_resume(lua_State *L, int nargs)".to_string()),
            "lua_pcall" => Some("int lua_pcall(lua_State *L, int nargs, int nresults, int errfunc)".to_string()),
            "lua_call" => Some("void lua_call(lua_State *L, int nargs, int nresults)".to_string()),
            _ => None,
        }
    }

    fn cross_reference_validate(&self, results: &mut Vec<FinderResult>) {
        let mut validated = vec![true; results.len()];

        for (i, result) in results.iter().enumerate() {
            if result.confidence < 0.8 {
                let mut cross_refs = 0;

                for (j, other) in results.iter().enumerate() {
                    if i != j && other.confidence >= 0.9 {
                        if let Some(analyzer) = &self.xref_analyzer {
                            let refs = analyzer.get_references_to(result.address);
                            if refs.iter().any(|r| self.find_function_start(*r) == other.address) {
                                cross_refs += 1;
                            }
                        }
                    }
                }

                if cross_refs == 0 && result.confidence < 0.75 {
                    validated[i] = false;
                }
            }
        }

        let mut i = 0;
        results.retain(|_| {
            let keep = validated[i];
            i += 1;
            keep
        });
    }
}

pub fn find_all_lua_api(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<FinderResult> {
    let mut finder = LuaApiFinder::new(reader);
    finder.find_all(start, end)
}
