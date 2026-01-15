// Tue Jan 13 2026 - Alex

use crate::finders::result::FinderResults;
use crate::validation::report::ValidationIssue;

pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue>;
}

pub struct RuleBuilder {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl RuleBuilder {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn add_address_range_rule(self, min: u64, max: u64) -> Self {
        self.add_rule(Box::new(CustomAddressRangeRule { min, max }))
    }

    pub fn add_alignment_rule(self, alignment: u64) -> Self {
        self.add_rule(Box::new(CustomAlignmentRule { alignment }))
    }

    pub fn add_offset_range_rule(self, max_offset: u64) -> Self {
        self.add_rule(Box::new(CustomOffsetRangeRule { max_offset }))
    }

    pub fn build(self) -> Vec<Box<dyn ValidationRule>> {
        self.rules
    }
}

impl Default for RuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct CustomAddressRangeRule {
    min: u64,
    max: u64,
}

impl ValidationRule for CustomAddressRangeRule {
    fn name(&self) -> &str {
        "CustomAddressRange"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (name, addr) in &results.functions {
            let a = addr.as_u64();
            if a != 0 && (a < self.min || a > self.max) {
                issues.push(ValidationIssue {
                    category: "address_range".to_string(),
                    item_name: name.clone(),
                    message: format!("Address 0x{:X} outside range 0x{:X}-0x{:X}", a, self.min, self.max),
                    severity: crate::validation::report::IssueSeverity::Warning,
                    suggestion: None,
                });
            }
        }

        issues
    }
}

struct CustomAlignmentRule {
    alignment: u64,
}

impl ValidationRule for CustomAlignmentRule {
    fn name(&self) -> &str {
        "CustomAlignment"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (name, addr) in &results.functions {
            if addr.as_u64() % self.alignment != 0 {
                issues.push(ValidationIssue {
                    category: "alignment".to_string(),
                    item_name: name.clone(),
                    message: format!("Address not {}-byte aligned", self.alignment),
                    severity: crate::validation::report::IssueSeverity::Warning,
                    suggestion: None,
                });
            }
        }

        issues
    }
}

struct CustomOffsetRangeRule {
    max_offset: u64,
}

impl ValidationRule for CustomOffsetRangeRule {
    fn name(&self) -> &str {
        "CustomOffsetRange"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                if *offset > self.max_offset {
                    issues.push(ValidationIssue {
                        category: "offset_range".to_string(),
                        item_name: format!("{}.{}", struct_name, field),
                        message: format!("Offset 0x{:X} exceeds max 0x{:X}", offset, self.max_offset),
                        severity: crate::validation::report::IssueSeverity::Warning,
                        suggestion: None,
                    });
                }
            }
        }

        issues
    }
}

pub struct LuaStateRules;

impl LuaStateRules {
    pub fn create_rules() -> Vec<Box<dyn ValidationRule>> {
        vec![
            Box::new(LuaStateOffsetRule),
        ]
    }
}

struct LuaStateOffsetRule;

impl ValidationRule for LuaStateOffsetRule {
    fn name(&self) -> &str {
        "LuaStateOffset"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(lua_state_offsets) = results.structure_offsets.get("lua_State") {
            if let Some(&top) = lua_state_offsets.get("top") {
                if let Some(&base) = lua_state_offsets.get("base") {
                    if top <= base {
                        issues.push(ValidationIssue {
                            category: "lua_state".to_string(),
                            item_name: "top/base".to_string(),
                            message: "top offset should be greater than base offset".to_string(),
                            severity: crate::validation::report::IssueSeverity::Error,
                            suggestion: Some("Check lua_State structure layout".to_string()),
                        });
                    }
                }
            }
        }

        issues
    }
}

pub struct RobloxRules;

impl RobloxRules {
    pub fn create_rules() -> Vec<Box<dyn ValidationRule>> {
        vec![
            Box::new(IdentityOffsetRule),
            Box::new(CapabilitiesOffsetRule),
        ]
    }
}

struct IdentityOffsetRule;

impl ValidationRule for IdentityOffsetRule {
    fn name(&self) -> &str {
        "IdentityOffset"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (struct_name, fields) in &results.structure_offsets {
            if struct_name.to_lowercase().contains("extra") || struct_name.to_lowercase().contains("sctx") {
                if let Some(&identity) = fields.get("identity") {
                    if identity > 0x100 {
                        issues.push(ValidationIssue {
                            category: "roblox".to_string(),
                            item_name: format!("{}.identity", struct_name),
                            message: format!("Identity offset 0x{:X} seems too large", identity),
                            severity: crate::validation::report::IssueSeverity::Warning,
                            suggestion: Some("Expected identity offset < 0x100".to_string()),
                        });
                    }
                }
            }
        }

        issues
    }
}

struct CapabilitiesOffsetRule;

impl ValidationRule for CapabilitiesOffsetRule {
    fn name(&self) -> &str {
        "CapabilitiesOffset"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (struct_name, fields) in &results.structure_offsets {
            if let Some(&caps) = fields.get("capabilities") {
                if let Some(&identity) = fields.get("identity") {
                    if caps < identity {
                        issues.push(ValidationIssue {
                            category: "roblox".to_string(),
                            item_name: format!("{}.capabilities", struct_name),
                            message: "Capabilities offset should be after identity".to_string(),
                            severity: crate::validation::report::IssueSeverity::Warning,
                            suggestion: None,
                        });
                    }
                }
            }
        }

        issues
    }
}
