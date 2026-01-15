// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::finders::result::FinderResults;
use crate::validation::rules::{ValidationRule, ValidationRuleSet};
use crate::validation::confidence::{ConfidenceScore, ConfidenceCalculator};
use crate::validation::report::ValidationReport;
use std::sync::Arc;
use std::collections::HashMap;

pub struct OffsetValidator {
    reader: Arc<dyn MemoryReader>,
    rule_set: ValidationRuleSet,
    confidence_calculator: ConfidenceCalculator,
}

impl OffsetValidator {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            rule_set: ValidationRuleSet::default(),
            confidence_calculator: ConfidenceCalculator::new(),
        }
    }

    pub fn with_rule_set(mut self, rule_set: ValidationRuleSet) -> Self {
        self.rule_set = rule_set;
        self
    }

    pub fn validate_all(&self, results: &FinderResults) -> ValidationReport {
        let mut report = ValidationReport::new();

        for (name, addr) in &results.functions {
            let result = self.validate_function(name, *addr);
            report.add_function_result(name.clone(), result);
        }

        for (struct_name, offsets) in &results.structure_offsets {
            for (field_name, offset) in offsets {
                let result = self.validate_structure_offset(struct_name, field_name, *offset);
                report.add_structure_result(struct_name.clone(), field_name.clone(), result);
            }
        }

        for (name, addr) in &results.classes {
            let result = self.validate_class(name, *addr);
            report.add_class_result(name.clone(), result);
        }

        for (name, value) in &results.constants {
            let result = self.validate_constant(name, *value);
            report.add_constant_result(name.clone(), result);
        }

        report.calculate_summary();
        report
    }

    pub fn validate_function(&self, name: &str, addr: Address) -> ValidationResult {
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        if addr.as_u64() == 0 {
            issues.push(ValidationIssue::NullAddress);
            confidence = 0.0;
            return ValidationResult {
                valid: false,
                confidence,
                issues,
            };
        }

        if addr.as_u64() % 4 != 0 {
            issues.push(ValidationIssue::MisalignedAddress);
            confidence *= 0.5;
        }

        if let Ok(regions) = self.reader.get_regions() {
            let in_executable = regions.iter()
                .any(|r| r.protection.is_executable() &&
                     addr.as_u64() >= r.range.start.as_u64() &&
                     addr.as_u64() < r.range.end.as_u64());

            if !in_executable {
                issues.push(ValidationIssue::NotInExecutableRegion);
                confidence *= 0.3;
            }
        }

        if let Ok(bytes) = self.reader.read_bytes(addr, 8) {
            if !self.looks_like_function_prologue(&bytes) {
                issues.push(ValidationIssue::InvalidFunctionPrologue);
                confidence *= 0.7;
            }
        }

        if let Some(rule) = self.rule_set.get_function_rule(name) {
            if let Some(expected_range) = &rule.expected_range {
                if addr.as_u64() < expected_range.0 || addr.as_u64() > expected_range.1 {
                    issues.push(ValidationIssue::OutOfExpectedRange);
                    confidence *= 0.6;
                }
            }
        }

        ValidationResult {
            valid: issues.is_empty() || confidence > 0.5,
            confidence,
            issues,
        }
    }

    pub fn validate_structure_offset(&self, struct_name: &str, field_name: &str, offset: u64) -> ValidationResult {
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        if offset % 8 != 0 && offset % 4 != 0 && offset % 2 != 0 {
            if !self.is_known_byte_field(struct_name, field_name) {
                issues.push(ValidationIssue::UnalignedOffset);
                confidence *= 0.8;
            }
        }

        if let Some(rule) = self.rule_set.get_structure_rule(struct_name, field_name) {
            if let Some(expected) = rule.expected_offset {
                if offset != expected {
                    if (offset as i64 - expected as i64).abs() < 0x10 {
                        issues.push(ValidationIssue::OffsetSlightlyDifferent);
                        confidence *= 0.9;
                    } else {
                        issues.push(ValidationIssue::OffsetSignificantlyDifferent);
                        confidence *= 0.5;
                    }
                }
            }

            if let Some(max) = rule.max_offset {
                if offset > max {
                    issues.push(ValidationIssue::OffsetTooLarge);
                    confidence *= 0.4;
                }
            }
        }

        if let Some(struct_size) = self.get_expected_struct_size(struct_name) {
            if offset >= struct_size {
                issues.push(ValidationIssue::OffsetBeyondStructSize);
                confidence *= 0.3;
            }
        }

        ValidationResult {
            valid: issues.is_empty() || confidence > 0.5,
            confidence,
            issues,
        }
    }

    pub fn validate_class(&self, name: &str, addr: Address) -> ValidationResult {
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        if addr.as_u64() == 0 {
            issues.push(ValidationIssue::NullAddress);
            confidence = 0.0;
            return ValidationResult {
                valid: false,
                confidence,
                issues,
            };
        }

        if addr.as_u64() % 8 != 0 {
            issues.push(ValidationIssue::MisalignedAddress);
            confidence *= 0.6;
        }

        if let Ok(bytes) = self.reader.read_bytes(addr, 64) {
            if !self.looks_like_vtable(&bytes) {
                issues.push(ValidationIssue::InvalidVTable);
                confidence *= 0.5;
            }
        }

        ValidationResult {
            valid: issues.is_empty() || confidence > 0.5,
            confidence,
            issues,
        }
    }

    pub fn validate_constant(&self, name: &str, value: u64) -> ValidationResult {
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        if let Some(rule) = self.rule_set.get_constant_rule(name) {
            if let Some(expected) = rule.expected_value {
                if value != expected {
                    issues.push(ValidationIssue::UnexpectedConstantValue);
                    confidence *= 0.5;
                }
            }

            if let Some((min, max)) = rule.value_range {
                if value < min || value > max {
                    issues.push(ValidationIssue::ConstantOutOfRange);
                    confidence *= 0.6;
                }
            }
        }

        ValidationResult {
            valid: issues.is_empty() || confidence > 0.5,
            confidence,
            issues,
        }
    }

    fn looks_like_function_prologue(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 4 {
            return false;
        }

        let first_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let is_stp_x29_x30 = (first_word & 0xFFC07FFF) == 0xA9007BFD;

        let is_sub_sp = (first_word & 0xFF0003FF) == 0xD10003FF;

        let is_stp_general = (first_word & 0xFE000000) == 0xA9000000;

        is_stp_x29_x30 || is_sub_sp || is_stp_general
    }

    fn looks_like_vtable(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 16 {
            return false;
        }

        let mut valid_pointers = 0;
        for i in 0..2 {
            let ptr = u64::from_le_bytes(bytes[i*8..(i+1)*8].try_into().unwrap_or([0; 8]));
            if ptr != 0 && ptr % 4 == 0 && ptr > 0x100000000 {
                valid_pointers += 1;
            }
        }

        valid_pointers >= 1
    }

    fn is_known_byte_field(&self, struct_name: &str, field_name: &str) -> bool {
        let lower_field = field_name.to_lowercase();

        lower_field.contains("type") ||
        lower_field.contains("flag") ||
        lower_field.contains("tag") ||
        lower_field.contains("marked")
    }

    fn get_expected_struct_size(&self, struct_name: &str) -> Option<u64> {
        match struct_name {
            "lua_State" => Some(0x100),
            "Closure" => Some(0x40),
            "Proto" => Some(0x80),
            "Table" => Some(0x50),
            "TValue" => Some(0x10),
            "GCObject" => Some(0x10),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub confidence: f64,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn passed() -> Self {
        Self {
            valid: true,
            confidence: 1.0,
            issues: Vec::new(),
        }
    }

    pub fn failed(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            confidence: 0.0,
            issues,
        }
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssue {
    NullAddress,
    MisalignedAddress,
    NotInExecutableRegion,
    InvalidFunctionPrologue,
    OutOfExpectedRange,
    UnalignedOffset,
    OffsetSlightlyDifferent,
    OffsetSignificantlyDifferent,
    OffsetTooLarge,
    OffsetBeyondStructSize,
    InvalidVTable,
    UnexpectedConstantValue,
    ConstantOutOfRange,
    FailedCrossValidation,
    ConflictingOffsets,
}

impl ValidationIssue {
    pub fn severity(&self) -> IssueSeverity {
        match self {
            ValidationIssue::NullAddress => IssueSeverity::Critical,
            ValidationIssue::MisalignedAddress => IssueSeverity::Warning,
            ValidationIssue::NotInExecutableRegion => IssueSeverity::Error,
            ValidationIssue::InvalidFunctionPrologue => IssueSeverity::Warning,
            ValidationIssue::OutOfExpectedRange => IssueSeverity::Warning,
            ValidationIssue::UnalignedOffset => IssueSeverity::Info,
            ValidationIssue::OffsetSlightlyDifferent => IssueSeverity::Info,
            ValidationIssue::OffsetSignificantlyDifferent => IssueSeverity::Warning,
            ValidationIssue::OffsetTooLarge => IssueSeverity::Error,
            ValidationIssue::OffsetBeyondStructSize => IssueSeverity::Error,
            ValidationIssue::InvalidVTable => IssueSeverity::Warning,
            ValidationIssue::UnexpectedConstantValue => IssueSeverity::Warning,
            ValidationIssue::ConstantOutOfRange => IssueSeverity::Error,
            ValidationIssue::FailedCrossValidation => IssueSeverity::Error,
            ValidationIssue::ConflictingOffsets => IssueSeverity::Critical,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ValidationIssue::NullAddress => "Address is null (0x0)",
            ValidationIssue::MisalignedAddress => "Address is not properly aligned",
            ValidationIssue::NotInExecutableRegion => "Address is not in an executable region",
            ValidationIssue::InvalidFunctionPrologue => "Does not appear to be a valid function prologue",
            ValidationIssue::OutOfExpectedRange => "Address is outside expected range",
            ValidationIssue::UnalignedOffset => "Offset is not naturally aligned",
            ValidationIssue::OffsetSlightlyDifferent => "Offset differs slightly from expected",
            ValidationIssue::OffsetSignificantlyDifferent => "Offset differs significantly from expected",
            ValidationIssue::OffsetTooLarge => "Offset exceeds maximum expected value",
            ValidationIssue::OffsetBeyondStructSize => "Offset extends beyond expected structure size",
            ValidationIssue::InvalidVTable => "Does not appear to be a valid vtable",
            ValidationIssue::UnexpectedConstantValue => "Constant value differs from expected",
            ValidationIssue::ConstantOutOfRange => "Constant value is outside valid range",
            ValidationIssue::FailedCrossValidation => "Cross-validation with related offsets failed",
            ValidationIssue::ConflictingOffsets => "Conflicting offset values detected",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
