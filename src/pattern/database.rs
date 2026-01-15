// Tue Jan 13 2026 - Alex

use crate::pattern::Pattern;
use std::collections::HashMap;

pub struct PatternDatabase {
    patterns: HashMap<String, PatternEntry>,
    categories: HashMap<String, Vec<String>>,
}

impl PatternDatabase {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn add_pattern(&mut self, name: &str, pattern: Pattern, category: &str) {
        let entry = PatternEntry {
            pattern,
            category: category.to_string(),
            description: None,
            version: None,
        };

        self.patterns.insert(name.to_string(), entry);

        self.categories.entry(category.to_string())
            .or_default()
            .push(name.to_string());
    }

    pub fn add_pattern_with_description(&mut self, name: &str, pattern: Pattern, category: &str, description: &str) {
        let entry = PatternEntry {
            pattern,
            category: category.to_string(),
            description: Some(description.to_string()),
            version: None,
        };

        self.patterns.insert(name.to_string(), entry);

        self.categories.entry(category.to_string())
            .or_default()
            .push(name.to_string());
    }

    pub fn get_pattern(&self, name: &str) -> Option<&Pattern> {
        self.patterns.get(name).map(|e| &e.pattern)
    }

    pub fn get_entry(&self, name: &str) -> Option<&PatternEntry> {
        self.patterns.get(name)
    }

    pub fn get_patterns_in_category(&self, category: &str) -> Vec<&Pattern> {
        self.categories.get(category)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.patterns.get(name))
                    .map(|entry| &entry.pattern)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_all_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PatternEntry)> {
        self.patterns.iter()
    }

    pub fn with_default_patterns() -> Self {
        let mut db = Self::new();

        db.add_pattern_with_description(
            "lua_newthread",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? ?? 94"),
            "lua_api",
            "Lua newthread function pattern"
        );

        db.add_pattern_with_description(
            "lua_pushstring",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 91 ?? ?? ?? 94"),
            "lua_api",
            "Lua pushstring function pattern"
        );

        db.add_pattern_with_description(
            "lua_pushnumber",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 1E ?? ?? ?? F9"),
            "lua_api",
            "Lua pushnumber function pattern"
        );

        db.add_pattern_with_description(
            "lua_pushboolean",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 52 ?? ?? ?? 72"),
            "lua_api",
            "Lua pushboolean function pattern"
        );

        db.add_pattern_with_description(
            "lua_pushnil",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? F9 ?? ?? ?? 52"),
            "lua_api",
            "Lua pushnil function pattern"
        );

        db.add_pattern_with_description(
            "lua_createtable",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? 2A ?? ?? ?? 94"),
            "lua_api",
            "Lua createtable function pattern"
        );

        db.add_pattern_with_description(
            "lua_settable",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? B9 ?? ?? ?? 94"),
            "lua_api",
            "Lua settable function pattern"
        );

        db.add_pattern_with_description(
            "lua_gettable",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? B9 ?? ?? ?? 94"),
            "lua_api",
            "Lua gettable function pattern"
        );

        db.add_pattern_with_description(
            "luau_load",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 F5 ?? ?? A9 F7 ?? ?? A9"),
            "roblox",
            "LuauLoad function pattern"
        );

        db.add_pattern_with_description(
            "push_instance",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 F5 ?? ?? A9 ?? ?? ?? F9"),
            "roblox",
            "PushInstance function pattern"
        );

        db.add_pattern_with_description(
            "task_scheduler",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 90 ?? ?? ?? F9 ?? ?? ?? B4"),
            "roblox",
            "TaskScheduler getter pattern"
        );

        db.add_pattern_with_description(
            "identity_check",
            Pattern::from_hex("B9 ?? ?? ?? 71 ?? ?? ?? 54 ?? ?? ?? B9"),
            "roblox",
            "Identity check pattern"
        );

        db.add_pattern_with_description(
            "arm64_prologue_stp",
            Pattern::from_hex("FD 7B ?? A9"),
            "arm64",
            "ARM64 function prologue (STP X29, X30, [SP, #imm]!)"
        );

        db.add_pattern_with_description(
            "arm64_prologue_stp_pair",
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91"),
            "arm64",
            "ARM64 function prologue with frame setup"
        );

        db.add_pattern_with_description(
            "arm64_epilogue_ret",
            Pattern::from_hex("C0 03 5F D6"),
            "arm64",
            "ARM64 RET instruction"
        );

        db.add_pattern_with_description(
            "arm64_bl",
            Pattern::from_hex("?? ?? ?? 94"),
            "arm64",
            "ARM64 BL (branch with link) instruction"
        );

        db.add_pattern_with_description(
            "vtable_reference",
            Pattern::from_hex("?? ?? ?? 90 ?? ?? ?? 91 ?? ?? ?? F9"),
            "class",
            "VTable reference pattern (ADRP + ADD + LDR)"
        );

        db.add_pattern_with_description(
            "string_reference",
            Pattern::from_hex("?? ?? ?? 90 ?? ?? ?? 91"),
            "class",
            "String constant reference pattern (ADRP + ADD)"
        );

        db
    }
}

impl Default for PatternDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PatternEntry {
    pub pattern: Pattern,
    pub category: String,
    pub description: Option<String>,
    pub version: Option<String>,
}
