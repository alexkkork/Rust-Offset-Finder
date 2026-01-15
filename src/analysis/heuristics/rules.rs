// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::heuristics::engine::HeuristicMatch;
use std::collections::HashMap;

pub trait HeuristicRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch>;
    
    fn description(&self) -> &str {
        ""
    }

    fn priority(&self) -> u32 {
        100
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Generic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Generic,
    FunctionDetection,
    StructureDetection,
    PatternRecognition,
    DataFlow,
    ControlFlow,
    StringAnalysis,
    PointerAnalysis,
}

impl RuleCategory {
    pub fn name(&self) -> &'static str {
        match self {
            RuleCategory::Generic => "Generic",
            RuleCategory::FunctionDetection => "Function Detection",
            RuleCategory::StructureDetection => "Structure Detection",
            RuleCategory::PatternRecognition => "Pattern Recognition",
            RuleCategory::DataFlow => "Data Flow",
            RuleCategory::ControlFlow => "Control Flow",
            RuleCategory::StringAnalysis => "String Analysis",
            RuleCategory::PointerAnalysis => "Pointer Analysis",
        }
    }
}

pub struct RuleEngine {
    rules: Vec<Box<dyn HeuristicRule>>,
    enabled_categories: Vec<RuleCategory>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            enabled_categories: vec![
                RuleCategory::Generic,
                RuleCategory::FunctionDetection,
                RuleCategory::StructureDetection,
                RuleCategory::PatternRecognition,
                RuleCategory::DataFlow,
                RuleCategory::ControlFlow,
                RuleCategory::StringAnalysis,
                RuleCategory::PointerAnalysis,
            ],
        }
    }

    pub fn add_rule(&mut self, rule: Box<dyn HeuristicRule>) {
        self.rules.push(rule);
    }

    pub fn enable_category(&mut self, category: RuleCategory) {
        if !self.enabled_categories.contains(&category) {
            self.enabled_categories.push(category);
        }
    }

    pub fn disable_category(&mut self, category: RuleCategory) {
        self.enabled_categories.retain(|c| *c != category);
    }

    pub fn check_all(&self, data: &[u8], addr: Address) -> Vec<HeuristicMatch> {
        let mut matches = Vec::new();

        for rule in &self.rules {
            if self.enabled_categories.contains(&rule.category()) {
                if let Some(m) = rule.check(data, addr) {
                    matches.push(m);
                }
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn rules_in_category(&self, category: RuleCategory) -> usize {
        self.rules.iter().filter(|r| r.category() == category).count()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LuaStateAccessRule {
    expected_offsets: HashMap<String, u64>,
}

impl LuaStateAccessRule {
    pub fn new() -> Self {
        let mut expected_offsets = HashMap::new();
        expected_offsets.insert("top".to_string(), 0x10);
        expected_offsets.insert("base".to_string(), 0x08);
        expected_offsets.insert("stack".to_string(), 0x18);
        expected_offsets.insert("ci".to_string(), 0x28);
        expected_offsets.insert("global".to_string(), 0x40);

        Self { expected_offsets }
    }
}

impl Default for LuaStateAccessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRule for LuaStateAccessRule {
    fn name(&self) -> &str {
        "LuaStateAccess"
    }

    fn description(&self) -> &str {
        "Detects access patterns to lua_State structure fields"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::StructureDetection
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst >> 22) == 0x3E5 {
            let imm = ((inst >> 10) & 0xFFF) * 8;
            let rn = (inst >> 5) & 0x1F;

            if rn == 0 {
                for (field, &expected) in &self.expected_offsets {
                    if imm as u64 == expected {
                        return Some(HeuristicMatch {
                            rule: self.name().to_string(),
                            address: addr,
                            confidence: 0.8,
                            description: format!("Potential lua_State.{} access at offset 0x{:X}", field, imm),
                        });
                    }
                }
            }
        }

        None
    }
}

pub struct ExtraSpaceAccessRule {
    extraspace_offset: u64,
    expected_fields: HashMap<String, u64>,
}

impl ExtraSpaceAccessRule {
    pub fn new() -> Self {
        let mut expected_fields = HashMap::new();
        expected_fields.insert("identity".to_string(), 0x08);
        expected_fields.insert("capabilities".to_string(), 0x10);
        expected_fields.insert("script_context".to_string(), 0x18);

        Self {
            extraspace_offset: 0x70,
            expected_fields,
        }
    }
}

impl Default for ExtraSpaceAccessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRule for ExtraSpaceAccessRule {
    fn name(&self) -> &str {
        "ExtraSpaceAccess"
    }

    fn description(&self) -> &str {
        "Detects access patterns to Roblox ExtraSpace/ScriptContext"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::StructureDetection
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 8 {
            return None;
        }

        let inst0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let inst1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        if (inst0 >> 22) == 0x3E5 {
            let offset0 = ((inst0 >> 10) & 0xFFF) * 8;

            if offset0 as u64 == self.extraspace_offset {
                if (inst1 >> 22) == 0x3E5 {
                    let offset1 = ((inst1 >> 10) & 0xFFF) * 8;

                    for (field, &expected) in &self.expected_fields {
                        if offset1 as u64 == expected {
                            return Some(HeuristicMatch {
                                rule: self.name().to_string(),
                                address: addr,
                                confidence: 0.85,
                                description: format!("ExtraSpace.{} access pattern", field),
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

pub struct ClosureAccessRule {
    expected_offsets: HashMap<String, u64>,
}

impl ClosureAccessRule {
    pub fn new() -> Self {
        let mut expected_offsets = HashMap::new();
        expected_offsets.insert("proto".to_string(), 0x20);
        expected_offsets.insert("env".to_string(), 0x18);
        expected_offsets.insert("nupvalues".to_string(), 0x09);

        Self { expected_offsets }
    }
}

impl Default for ClosureAccessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRule for ClosureAccessRule {
    fn name(&self) -> &str {
        "ClosureAccess"
    }

    fn description(&self) -> &str {
        "Detects access patterns to Closure structure fields"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::StructureDetection
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst >> 22) == 0x3E5 {
            let imm = ((inst >> 10) & 0xFFF) * 8;

            for (field, &expected) in &self.expected_offsets {
                if imm as u64 == expected {
                    return Some(HeuristicMatch {
                        rule: self.name().to_string(),
                        address: addr,
                        confidence: 0.7,
                        description: format!("Potential Closure.{} access", field),
                    });
                }
            }
        }

        None
    }
}

pub struct ProtoAccessRule {
    expected_offsets: HashMap<String, u64>,
}

impl ProtoAccessRule {
    pub fn new() -> Self {
        let mut expected_offsets = HashMap::new();
        expected_offsets.insert("code".to_string(), 0x20);
        expected_offsets.insert("k".to_string(), 0x28);
        expected_offsets.insert("sizecode".to_string(), 0x10);
        expected_offsets.insert("sizek".to_string(), 0x14);
        expected_offsets.insert("source".to_string(), 0x60);

        Self { expected_offsets }
    }
}

impl Default for ProtoAccessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRule for ProtoAccessRule {
    fn name(&self) -> &str {
        "ProtoAccess"
    }

    fn description(&self) -> &str {
        "Detects access patterns to Proto structure fields"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::StructureDetection
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst >> 22) == 0x3E5 {
            let imm = ((inst >> 10) & 0xFFF) * 8;

            for (field, &expected) in &self.expected_offsets {
                if imm as u64 == expected {
                    return Some(HeuristicMatch {
                        rule: self.name().to_string(),
                        address: addr,
                        confidence: 0.7,
                        description: format!("Potential Proto.{} access", field),
                    });
                }
            }
        }

        None
    }
}

pub struct TableAccessRule {
    expected_offsets: HashMap<String, u64>,
}

impl TableAccessRule {
    pub fn new() -> Self {
        let mut expected_offsets = HashMap::new();
        expected_offsets.insert("array".to_string(), 0x18);
        expected_offsets.insert("node".to_string(), 0x20);
        expected_offsets.insert("metatable".to_string(), 0x28);
        expected_offsets.insert("sizearray".to_string(), 0x28);

        Self { expected_offsets }
    }
}

impl Default for TableAccessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicRule for TableAccessRule {
    fn name(&self) -> &str {
        "TableAccess"
    }

    fn description(&self) -> &str {
        "Detects access patterns to Table structure fields"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::StructureDetection
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst >> 22) == 0x3E5 {
            let imm = ((inst >> 10) & 0xFFF) * 8;

            for (field, &expected) in &self.expected_offsets {
                if imm as u64 == expected {
                    return Some(HeuristicMatch {
                        rule: self.name().to_string(),
                        address: addr,
                        confidence: 0.65,
                        description: format!("Potential Table.{} access", field),
                    });
                }
            }
        }

        None
    }
}

pub fn create_default_rules() -> Vec<Box<dyn HeuristicRule>> {
    vec![
        Box::new(LuaStateAccessRule::new()),
        Box::new(ExtraSpaceAccessRule::new()),
        Box::new(ClosureAccessRule::new()),
        Box::new(ProtoAccessRule::new()),
        Box::new(TableAccessRule::new()),
    ]
}
