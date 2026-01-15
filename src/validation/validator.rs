// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::FinderResults;
use crate::validation::rules::ValidationRule;
use crate::validation::checker::ValidationChecker;
use crate::validation::report::{ValidationReport, ValidationIssue, IssueSeverity};
use crate::validation::confidence::ConfidenceScorer;
use std::sync::Arc;
use std::collections::HashMap;

pub struct OffsetValidator {
    reader: Arc<dyn MemoryReader>,
    rules: Vec<Box<dyn ValidationRule>>,
    checker: ValidationChecker,
    scorer: ConfidenceScorer,
}

impl OffsetValidator {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            rules: Self::default_rules(),
            checker: ValidationChecker::new(reader),
            scorer: ConfidenceScorer::new(),
        }
    }

    fn default_rules() -> Vec<Box<dyn ValidationRule>> {
        vec![
            Box::new(AddressRangeRule::new()),
            Box::new(AlignmentRule::new()),
            Box::new(OffsetSizeRule::new()),
            Box::new(DuplicateRule::new()),
        ]
    }

    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    pub fn validate(&self, results: &FinderResults) -> ValidationReport {
        let mut report = ValidationReport::new();

        for (name, addr) in &results.functions {
            self.validate_function(&mut report, name, *addr);
        }

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                self.validate_offset(&mut report, struct_name, field, *offset);
            }
        }

        for (name, addr) in &results.classes {
            self.validate_class(&mut report, name, *addr);
        }

        for rule in &self.rules {
            let issues = rule.validate_all(results);
            report.add_issues(issues);
        }

        report.calculate_overall_score();
        report
    }

    fn validate_function(&self, report: &mut ValidationReport, name: &str, addr: Address) {
        if addr.as_u64() == 0 {
            report.add_issue(ValidationIssue {
                category: "function".to_string(),
                item_name: name.to_string(),
                message: "Function address is null".to_string(),
                severity: IssueSeverity::Error,
                suggestion: Some("Re-scan for function or verify binary is loaded".to_string()),
            });
            return;
        }

        if addr.as_u64() % 4 != 0 {
            report.add_issue(ValidationIssue {
                category: "function".to_string(),
                item_name: name.to_string(),
                message: format!("Function address 0x{:X} is not 4-byte aligned", addr.as_u64()),
                severity: IssueSeverity::Warning,
                suggestion: Some("ARM64 functions should be 4-byte aligned".to_string()),
            });
        }

        if self.checker.check_function_prologue(addr).is_err() {
            report.add_issue(ValidationIssue {
                category: "function".to_string(),
                item_name: name.to_string(),
                message: "Could not verify function prologue".to_string(),
                severity: IssueSeverity::Info,
                suggestion: Some("Address might not point to function start".to_string()),
            });
        }
    }

    fn validate_offset(&self, report: &mut ValidationReport, struct_name: &str, field: &str, offset: u64) {
        if offset > 0x10000 {
            report.add_issue(ValidationIssue {
                category: "structure".to_string(),
                item_name: format!("{}.{}", struct_name, field),
                message: format!("Offset 0x{:X} seems unusually large", offset),
                severity: IssueSeverity::Warning,
                suggestion: Some("Verify this offset is correct".to_string()),
            });
        }

        if offset % 8 != 0 && offset % 4 != 0 && offset % 2 != 0 {
            let expected_alignment = if field.contains("ptr") || field.contains("Ptr") {
                8
            } else {
                4
            };
            
            report.add_issue(ValidationIssue {
                category: "structure".to_string(),
                item_name: format!("{}.{}", struct_name, field),
                message: format!("Offset 0x{:X} may not be properly aligned", offset),
                severity: IssueSeverity::Info,
                suggestion: Some(format!("Expected alignment: {} bytes", expected_alignment)),
            });
        }
    }

    fn validate_class(&self, report: &mut ValidationReport, name: &str, addr: Address) {
        if addr.as_u64() == 0 {
            report.add_issue(ValidationIssue {
                category: "class".to_string(),
                item_name: name.to_string(),
                message: "Class address is null".to_string(),
                severity: IssueSeverity::Error,
                suggestion: None,
            });
            return;
        }

        if addr.as_u64() % 8 != 0 {
            report.add_issue(ValidationIssue {
                category: "class".to_string(),
                item_name: name.to_string(),
                message: format!("Class/VTable address 0x{:X} is not 8-byte aligned", addr.as_u64()),
                severity: IssueSeverity::Warning,
                suggestion: Some("VTables should be 8-byte aligned on ARM64".to_string()),
            });
        }
    }

    pub fn get_confidence_scores(&self, results: &FinderResults) -> HashMap<String, f64> {
        self.scorer.calculate_all(results)
    }
}

struct AddressRangeRule {
    min_address: u64,
    max_address: u64,
}

impl AddressRangeRule {
    fn new() -> Self {
        Self {
            min_address: 0x100000000,
            max_address: 0x800000000000,
        }
    }
}

impl ValidationRule for AddressRangeRule {
    fn name(&self) -> &str {
        "AddressRange"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (name, addr) in &results.functions {
            let a = addr.as_u64();
            if a != 0 && (a < self.min_address || a > self.max_address) {
                issues.push(ValidationIssue {
                    category: "address_range".to_string(),
                    item_name: name.clone(),
                    message: format!("Address 0x{:X} is outside expected range", a),
                    severity: IssueSeverity::Warning,
                    suggestion: Some(format!("Expected range: 0x{:X} - 0x{:X}", self.min_address, self.max_address)),
                });
            }
        }

        issues
    }
}

struct AlignmentRule;

impl AlignmentRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for AlignmentRule {
    fn name(&self) -> &str {
        "Alignment"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (name, addr) in &results.functions {
            if addr.as_u64() % 4 != 0 {
                issues.push(ValidationIssue {
                    category: "alignment".to_string(),
                    item_name: name.clone(),
                    message: "Function not 4-byte aligned".to_string(),
                    severity: IssueSeverity::Warning,
                    suggestion: None,
                });
            }
        }

        issues
    }
}

struct OffsetSizeRule {
    max_reasonable_offset: u64,
}

impl OffsetSizeRule {
    fn new() -> Self {
        Self {
            max_reasonable_offset: 0x10000,
        }
    }
}

impl ValidationRule for OffsetSizeRule {
    fn name(&self) -> &str {
        "OffsetSize"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                if *offset > self.max_reasonable_offset {
                    issues.push(ValidationIssue {
                        category: "offset_size".to_string(),
                        item_name: format!("{}.{}", struct_name, field),
                        message: format!("Offset 0x{:X} exceeds reasonable maximum", offset),
                        severity: IssueSeverity::Warning,
                        suggestion: Some(format!("Expected max offset: 0x{:X}", self.max_reasonable_offset)),
                    });
                }
            }
        }

        issues
    }
}

struct DuplicateRule;

impl DuplicateRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for DuplicateRule {
    fn name(&self) -> &str {
        "Duplicates"
    }

    fn validate_all(&self, results: &FinderResults) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut seen_addresses: HashMap<u64, Vec<String>> = HashMap::new();

        for (name, addr) in &results.functions {
            seen_addresses.entry(addr.as_u64())
                .or_default()
                .push(name.clone());
        }

        for (addr, names) in seen_addresses {
            if names.len() > 1 && addr != 0 {
                issues.push(ValidationIssue {
                    category: "duplicates".to_string(),
                    item_name: names.join(", "),
                    message: format!("Multiple functions point to same address 0x{:X}", addr),
                    severity: IssueSeverity::Info,
                    suggestion: Some("These may be aliases or one may be incorrect".to_string()),
                });
            }
        }

        issues
    }
}
