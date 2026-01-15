// Tue Jan 13 2026 - Alex

use std::collections::HashMap;

pub struct ValidationRuleSet {
    function_rules: HashMap<String, FunctionRule>,
    structure_rules: HashMap<String, HashMap<String, StructureFieldRule>>,
    constant_rules: HashMap<String, ConstantRule>,
    global_rules: Vec<GlobalRule>,
}

impl ValidationRuleSet {
    pub fn new() -> Self {
        Self {
            function_rules: HashMap::new(),
            structure_rules: HashMap::new(),
            constant_rules: HashMap::new(),
            global_rules: Vec::new(),
        }
    }

    pub fn add_function_rule(&mut self, name: &str, rule: FunctionRule) {
        self.function_rules.insert(name.to_string(), rule);
    }

    pub fn add_structure_rule(&mut self, struct_name: &str, field_name: &str, rule: StructureFieldRule) {
        self.structure_rules
            .entry(struct_name.to_string())
            .or_default()
            .insert(field_name.to_string(), rule);
    }

    pub fn add_constant_rule(&mut self, name: &str, rule: ConstantRule) {
        self.constant_rules.insert(name.to_string(), rule);
    }

    pub fn add_global_rule(&mut self, rule: GlobalRule) {
        self.global_rules.push(rule);
    }

    pub fn get_function_rule(&self, name: &str) -> Option<&FunctionRule> {
        self.function_rules.get(name)
    }

    pub fn get_structure_rule(&self, struct_name: &str, field_name: &str) -> Option<&StructureFieldRule> {
        self.structure_rules
            .get(struct_name)
            .and_then(|fields| fields.get(field_name))
    }

    pub fn get_constant_rule(&self, name: &str) -> Option<&ConstantRule> {
        self.constant_rules.get(name)
    }

    pub fn global_rules(&self) -> &[GlobalRule] {
        &self.global_rules
    }
}

impl Default for ValidationRuleSet {
    fn default() -> Self {
        let mut rules = Self::new();

        rules.add_function_rule("luau_load", FunctionRule {
            expected_range: None,
            required_prologue: Some(PrologueType::Standard),
            must_call: vec![],
            must_reference: vec![],
        });

        rules.add_function_rule("lua_pushstring", FunctionRule {
            expected_range: None,
            required_prologue: Some(PrologueType::Standard),
            must_call: vec![],
            must_reference: vec![],
        });

        rules.add_structure_rule("lua_State", "top", StructureFieldRule {
            expected_offset: None,
            max_offset: Some(0x100),
            alignment: Some(8),
            field_type: FieldType::Pointer,
        });

        rules.add_structure_rule("lua_State", "stack", StructureFieldRule {
            expected_offset: None,
            max_offset: Some(0x100),
            alignment: Some(8),
            field_type: FieldType::Pointer,
        });

        rules.add_structure_rule("Closure", "proto", StructureFieldRule {
            expected_offset: None,
            max_offset: Some(0x40),
            alignment: Some(8),
            field_type: FieldType::Pointer,
        });

        rules.add_global_rule(GlobalRule::OffsetsMustBeUnique);
        rules.add_global_rule(GlobalRule::AddressesMustBeInRange);

        rules
    }
}

#[derive(Debug, Clone)]
pub struct FunctionRule {
    pub expected_range: Option<(u64, u64)>,
    pub required_prologue: Option<PrologueType>,
    pub must_call: Vec<String>,
    pub must_reference: Vec<String>,
}

impl FunctionRule {
    pub fn new() -> Self {
        Self {
            expected_range: None,
            required_prologue: None,
            must_call: Vec::new(),
            must_reference: Vec::new(),
        }
    }

    pub fn with_range(mut self, min: u64, max: u64) -> Self {
        self.expected_range = Some((min, max));
        self
    }

    pub fn with_prologue(mut self, prologue: PrologueType) -> Self {
        self.required_prologue = Some(prologue);
        self
    }

    pub fn must_call(mut self, function: &str) -> Self {
        self.must_call.push(function.to_string());
        self
    }

    pub fn must_reference(mut self, data: &str) -> Self {
        self.must_reference.push(data.to_string());
        self
    }
}

impl Default for FunctionRule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StructureFieldRule {
    pub expected_offset: Option<u64>,
    pub max_offset: Option<u64>,
    pub alignment: Option<u64>,
    pub field_type: FieldType,
}

impl StructureFieldRule {
    pub fn new(field_type: FieldType) -> Self {
        Self {
            expected_offset: None,
            max_offset: None,
            alignment: None,
            field_type,
        }
    }

    pub fn with_expected(mut self, offset: u64) -> Self {
        self.expected_offset = Some(offset);
        self
    }

    pub fn with_max(mut self, max: u64) -> Self {
        self.max_offset = Some(max);
        self
    }

    pub fn with_alignment(mut self, align: u64) -> Self {
        self.alignment = Some(align);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ConstantRule {
    pub expected_value: Option<u64>,
    pub value_range: Option<(u64, u64)>,
    pub must_be_aligned: Option<u64>,
}

impl ConstantRule {
    pub fn new() -> Self {
        Self {
            expected_value: None,
            value_range: None,
            must_be_aligned: None,
        }
    }

    pub fn with_expected(mut self, value: u64) -> Self {
        self.expected_value = Some(value);
        self
    }

    pub fn with_range(mut self, min: u64, max: u64) -> Self {
        self.value_range = Some((min, max));
        self
    }

    pub fn must_align(mut self, alignment: u64) -> Self {
        self.must_be_aligned = Some(alignment);
        self
    }
}

impl Default for ConstantRule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrologueType {
    Standard,
    LeafFunction,
    TailCallOptimized,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Pointer,
    Integer,
    Float,
    Byte,
    Array,
    Struct,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalRule {
    OffsetsMustBeUnique,
    AddressesMustBeInRange,
    StructureSizeMustMatch,
    CrossReferencesMustExist,
}

pub trait ValidationRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn validate(&self, value: u64, context: &ValidationContext) -> bool;
}

pub struct ValidationContext {
    pub all_functions: HashMap<String, u64>,
    pub all_offsets: HashMap<String, HashMap<String, u64>>,
    pub all_constants: HashMap<String, u64>,
    pub memory_regions: Vec<(u64, u64, bool)>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            all_functions: HashMap::new(),
            all_offsets: HashMap::new(),
            all_constants: HashMap::new(),
            memory_regions: Vec::new(),
        }
    }

    pub fn is_in_executable_region(&self, addr: u64) -> bool {
        self.memory_regions.iter()
            .any(|(start, end, exec)| *exec && addr >= *start && addr < *end)
    }

    pub fn is_in_any_region(&self, addr: u64) -> bool {
        self.memory_regions.iter()
            .any(|(start, end, _)| addr >= *start && addr < *end)
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AlignmentRule {
    alignment: u64,
}

impl AlignmentRule {
    pub fn new(alignment: u64) -> Self {
        Self { alignment }
    }
}

impl ValidationRule for AlignmentRule {
    fn name(&self) -> &str {
        "Alignment Check"
    }

    fn description(&self) -> &str {
        "Checks if value is properly aligned"
    }

    fn validate(&self, value: u64, _context: &ValidationContext) -> bool {
        value % self.alignment == 0
    }
}

pub struct RangeRule {
    min: u64,
    max: u64,
}

impl RangeRule {
    pub fn new(min: u64, max: u64) -> Self {
        Self { min, max }
    }
}

impl ValidationRule for RangeRule {
    fn name(&self) -> &str {
        "Range Check"
    }

    fn description(&self) -> &str {
        "Checks if value is within expected range"
    }

    fn validate(&self, value: u64, _context: &ValidationContext) -> bool {
        value >= self.min && value <= self.max
    }
}

pub struct ExecutableRegionRule;

impl ValidationRule for ExecutableRegionRule {
    fn name(&self) -> &str {
        "Executable Region Check"
    }

    fn description(&self) -> &str {
        "Checks if address is in an executable region"
    }

    fn validate(&self, value: u64, context: &ValidationContext) -> bool {
        context.is_in_executable_region(value)
    }
}
