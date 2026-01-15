// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::{ControlFlowGraph, BasicBlock, Instruction, HeuristicEngine};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ValidationEngine {
    reader: Arc<dyn MemoryReader>,
    validators: Vec<Box<dyn Validator>>,
    strictness: ValidationStrictness,
}

pub trait Validator: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, context: &ValidationContext) -> ValidationResult;
    fn severity(&self) -> ValidationSeverity { ValidationSeverity::Warning }
}

pub struct ValidationContext<'a> {
    pub reader: &'a dyn MemoryReader,
    pub address: Address,
    pub cfg: Option<&'a ControlFlowGraph>,
    pub expected_type: Option<ValidationType>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub confidence: f64,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub address: Address,
    pub message: String,
    pub severity: ValidationSeverity,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStrictness {
    Lenient,
    Normal,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationType {
    Function,
    VTable,
    String,
    Class,
    Data,
    Code,
}

impl ValidationEngine {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let mut engine = Self {
            reader,
            validators: Vec::new(),
            strictness: ValidationStrictness::Normal,
        };
        engine.register_default_validators();
        engine
    }

    pub fn with_strictness(mut self, strictness: ValidationStrictness) -> Self {
        self.strictness = strictness;
        self
    }

    fn register_default_validators(&mut self) {
        self.add_validator(Box::new(AddressRangeValidator));
        self.add_validator(Box::new(AlignmentValidator));
        self.add_validator(Box::new(InstructionValidator));
        self.add_validator(Box::new(ControlFlowValidator));
        self.add_validator(Box::new(PointerValidator));
        self.add_validator(Box::new(StringValidator));
        self.add_validator(Box::new(ConsistencyValidator));
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    pub fn validate(&self, address: Address, expected_type: Option<ValidationType>) -> ValidationResult {
        let context = ValidationContext {
            reader: self.reader.as_ref(),
            address,
            cfg: None,
            expected_type,
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_confidence = 0.0;
        let mut validator_count = 0;

        for validator in &self.validators {
            let result = validator.validate(&context);

            if !result.valid {
                for err in result.errors {
                    if self.should_report(&err) {
                        errors.push(err);
                    }
                }
            }

            warnings.extend(result.warnings);
            total_confidence += result.confidence;
            validator_count += 1;
        }

        let avg_confidence = if validator_count > 0 {
            total_confidence / validator_count as f64
        } else {
            0.0
        };

        let has_critical = errors.iter().any(|e| e.severity == ValidationSeverity::Critical);
        let has_errors = errors.iter().any(|e| e.severity == ValidationSeverity::Error);

        let valid = match self.strictness {
            ValidationStrictness::Lenient => !has_critical,
            ValidationStrictness::Normal => !has_critical && !has_errors,
            ValidationStrictness::Strict => errors.is_empty(),
        };

        ValidationResult {
            valid,
            confidence: avg_confidence,
            errors,
            warnings,
        }
    }

    pub fn validate_with_cfg(&self, address: Address, cfg: &ControlFlowGraph) -> ValidationResult {
        let context = ValidationContext {
            reader: self.reader.as_ref(),
            address,
            cfg: Some(cfg),
            expected_type: Some(ValidationType::Function),
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_confidence = 0.0;
        let mut validator_count = 0;

        for validator in &self.validators {
            let result = validator.validate(&context);

            if !result.valid {
                for err in result.errors {
                    if self.should_report(&err) {
                        errors.push(err);
                    }
                }
            }

            warnings.extend(result.warnings);
            total_confidence += result.confidence;
            validator_count += 1;
        }

        let avg_confidence = if validator_count > 0 {
            total_confidence / validator_count as f64
        } else {
            0.0
        };

        ValidationResult {
            valid: errors.iter().all(|e| e.severity != ValidationSeverity::Critical),
            confidence: avg_confidence,
            errors,
            warnings,
        }
    }

    fn should_report(&self, error: &ValidationError) -> bool {
        match self.strictness {
            ValidationStrictness::Lenient => {
                error.severity == ValidationSeverity::Critical
            }
            ValidationStrictness::Normal => {
                error.severity >= ValidationSeverity::Warning
            }
            ValidationStrictness::Strict => true,
        }
    }

    pub fn validate_offset(&self, name: &str, address: u64) -> OffsetValidation {
        let addr = Address::new(address);
        let result = self.validate(addr, None);

        OffsetValidation {
            name: name.to_string(),
            address,
            is_valid: result.valid,
            confidence: result.confidence,
            errors: result.errors.iter().map(|e| e.message.clone()).collect(),
        }
    }

    pub fn validate_function_offset(&self, name: &str, address: u64) -> OffsetValidation {
        let addr = Address::new(address);
        let result = self.validate(addr, Some(ValidationType::Function));

        OffsetValidation {
            name: name.to_string(),
            address,
            is_valid: result.valid,
            confidence: result.confidence,
            errors: result.errors.iter().map(|e| e.message.clone()).collect(),
        }
    }

    pub fn validate_all_offsets(&self, offsets: &[(String, u64)]) -> Vec<OffsetValidation> {
        offsets
            .iter()
            .map(|(name, addr)| self.validate_offset(name, *addr))
            .collect()
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

#[derive(Debug, Clone)]
pub struct OffsetValidation {
    pub name: String,
    pub address: u64,
    pub is_valid: bool,
    pub confidence: f64,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn success(confidence: f64) -> Self {
        Self {
            valid: true,
            confidence,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            confidence: 0.0,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn merge(&self, other: &ValidationResult) -> ValidationResult {
        let mut errors = self.errors.clone();
        errors.extend(other.errors.clone());

        let mut warnings = self.warnings.clone();
        warnings.extend(other.warnings.clone());

        ValidationResult {
            valid: self.valid && other.valid,
            confidence: (self.confidence + other.confidence) / 2.0,
            errors,
            warnings,
        }
    }
}

impl ValidationError {
    pub fn new(address: Address, message: String, severity: ValidationSeverity) -> Self {
        Self {
            address,
            message,
            severity,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

struct AddressRangeValidator;

impl Validator for AddressRangeValidator {
    fn name(&self) -> &str { "address_range" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        let addr = context.address.as_u64();

        if addr == 0 {
            return ValidationResult::failure(vec![
                ValidationError::new(context.address, "Null address".to_string(), ValidationSeverity::Critical)
            ]);
        }

        if addr < 0x100000000 {
            return ValidationResult::failure(vec![
                ValidationError::new(context.address, "Address too low for 64-bit".to_string(), ValidationSeverity::Error)
            ]);
        }

        if addr >= 0x800000000000 {
            return ValidationResult::failure(vec![
                ValidationError::new(context.address, "Address out of user space".to_string(), ValidationSeverity::Error)
            ]);
        }

        ValidationResult::success(0.9)
    }

    fn severity(&self) -> ValidationSeverity { ValidationSeverity::Critical }
}

struct AlignmentValidator;

impl Validator for AlignmentValidator {
    fn name(&self) -> &str { "alignment" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        let addr = context.address.as_u64();

        let required_alignment = match context.expected_type {
            Some(ValidationType::Function) => 4,
            Some(ValidationType::VTable) => 8,
            Some(ValidationType::String) => 1,
            _ => 1,
        };

        if addr % required_alignment != 0 {
            return ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    format!("Address not aligned to {} bytes", required_alignment),
                    ValidationSeverity::Warning,
                )
            ]);
        }

        ValidationResult::success(0.95)
    }
}

struct InstructionValidator;

impl Validator for InstructionValidator {
    fn name(&self) -> &str { "instruction" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        if context.expected_type != Some(ValidationType::Function) &&
           context.expected_type != Some(ValidationType::Code) {
            return ValidationResult::success(1.0);
        }

        match context.reader.read_u32(context.address) {
            Ok(insn) => {
                let op0 = (insn >> 25) & 0xF;
                if op0 == 0 || op0 == 1 || op0 == 3 {
                    return ValidationResult::failure(vec![
                        ValidationError::new(
                            context.address,
                            "Invalid instruction encoding".to_string(),
                            ValidationSeverity::Error,
                        )
                    ]);
                }
                ValidationResult::success(0.85)
            }
            Err(_) => ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    "Failed to read instruction".to_string(),
                    ValidationSeverity::Critical,
                )
            ]),
        }
    }
}

struct ControlFlowValidator;

impl Validator for ControlFlowValidator {
    fn name(&self) -> &str { "control_flow" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        let cfg = match context.cfg {
            Some(c) => c,
            None => return ValidationResult::success(1.0),
        };

        if cfg.block_count() == 0 {
            return ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    "Empty control flow graph".to_string(),
                    ValidationSeverity::Error,
                )
            ]);
        }

        if cfg.entry().is_none() {
            return ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    "No entry block".to_string(),
                    ValidationSeverity::Error,
                )
            ]);
        }

        if cfg.exits().is_empty() {
            return ValidationResult::success(0.7)
                .with_warning("No explicit exit blocks".to_string());
        }

        ValidationResult::success(0.9)
    }
}

struct PointerValidator;

impl Validator for PointerValidator {
    fn name(&self) -> &str { "pointer" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        if context.expected_type != Some(ValidationType::VTable) {
            return ValidationResult::success(1.0);
        }

        match context.reader.read_u64(context.address) {
            Ok(ptr) => {
                if ptr == 0 {
                    return ValidationResult::success(0.5)
                        .with_warning("Null pointer at vtable entry".to_string());
                }

                if ptr < 0x100000000 || ptr >= 0x800000000000 {
                    return ValidationResult::failure(vec![
                        ValidationError::new(
                            context.address,
                            "Invalid pointer value".to_string(),
                            ValidationSeverity::Error,
                        )
                    ]);
                }

                ValidationResult::success(0.9)
            }
            Err(_) => ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    "Failed to read pointer".to_string(),
                    ValidationSeverity::Critical,
                )
            ]),
        }
    }
}

struct StringValidator;

impl Validator for StringValidator {
    fn name(&self) -> &str { "string" }

    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        if context.expected_type != Some(ValidationType::String) {
            return ValidationResult::success(1.0);
        }

        match context.reader.read_bytes(context.address, 256) {
            Ok(bytes) => {
                let printable_count = bytes.iter()
                    .take_while(|&&b| b >= 0x20 && b < 0x7f || b == 0x09 || b == 0x0a || b == 0x0d)
                    .count();

                let has_null = bytes.iter().any(|&b| b == 0);

                if printable_count < 1 || !has_null {
                    return ValidationResult::failure(vec![
                        ValidationError::new(
                            context.address,
                            "Invalid string format".to_string(),
                            ValidationSeverity::Error,
                        )
                    ]);
                }

                let confidence = (printable_count as f64 / 20.0).min(0.95);
                ValidationResult::success(confidence)
            }
            Err(_) => ValidationResult::failure(vec![
                ValidationError::new(
                    context.address,
                    "Failed to read string data".to_string(),
                    ValidationSeverity::Critical,
                )
            ]),
        }
    }
}

struct ConsistencyValidator;

impl Validator for ConsistencyValidator {
    fn name(&self) -> &str { "consistency" }

    fn validate(&self, _context: &ValidationContext) -> ValidationResult {
        ValidationResult::success(1.0)
    }
}

pub fn quick_validate(reader: &dyn MemoryReader, address: Address) -> bool {
    if address.as_u64() == 0 || address.as_u64() < 0x100000000 {
        return false;
    }

    reader.read_u32(address).is_ok()
}

pub fn validate_pointer(reader: &dyn MemoryReader, address: Address) -> bool {
    match reader.read_u64(address) {
        Ok(ptr) => ptr >= 0x100000000 && ptr < 0x800000000000,
        Err(_) => false,
    }
}

pub fn validate_function_pointer(reader: &dyn MemoryReader, address: Address) -> bool {
    match reader.read_u64(address) {
        Ok(ptr) => {
            if ptr < 0x100000000 || ptr >= 0x800000000000 {
                return false;
            }

            let ptr_addr = Address::new(ptr);
            if let Ok(insn) = reader.read_u32(ptr_addr) {
                let op0 = (insn >> 25) & 0xF;
                return op0 != 0 && op0 != 1 && op0 != 3;
            }
            false
        }
        Err(_) => false,
    }
}
