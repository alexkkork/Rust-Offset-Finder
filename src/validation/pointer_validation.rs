// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryRegion, MemoryError};
use std::sync::Arc;
use std::collections::HashSet;
use std::fmt;

/// Configuration for pointer validation
#[derive(Debug, Clone)]
pub struct PointerValidationConfig {
    /// Minimum valid address
    pub min_address: u64,
    /// Maximum valid address
    pub max_address: u64,
    /// Required alignment for pointers
    pub alignment: usize,
    /// Whether to validate pointer targets
    pub validate_targets: bool,
    /// Maximum chain depth for pointer following
    pub max_chain_depth: usize,
    /// Whether to allow null pointers
    pub allow_null: bool,
    /// Known valid address ranges
    pub valid_ranges: Vec<(u64, u64)>,
}

impl Default for PointerValidationConfig {
    fn default() -> Self {
        Self {
            min_address: 0x100000000,
            max_address: 0x800000000000,
            alignment: 8,
            validate_targets: true,
            max_chain_depth: 5,
            allow_null: false,
            valid_ranges: Vec::new(),
        }
    }
}

impl PointerValidationConfig {
    pub fn relaxed() -> Self {
        Self {
            min_address: 0x1000,
            max_address: 0xFFFFFFFFFFFFFFFF,
            alignment: 1,
            validate_targets: false,
            max_chain_depth: 2,
            allow_null: true,
            valid_ranges: Vec::new(),
        }
    }

    pub fn strict() -> Self {
        Self {
            min_address: 0x100000000,
            max_address: 0x400000000000,
            alignment: 8,
            validate_targets: true,
            max_chain_depth: 10,
            allow_null: false,
            valid_ranges: Vec::new(),
        }
    }

    pub fn with_range(mut self, start: u64, end: u64) -> Self {
        self.valid_ranges.push((start, end));
        self
    }

    pub fn is_address_valid(&self, addr: u64) -> bool {
        if addr == 0 && self.allow_null {
            return true;
        }

        if addr < self.min_address || addr > self.max_address {
            return false;
        }

        if !self.valid_ranges.is_empty() {
            return self.valid_ranges.iter().any(|(start, end)| addr >= *start && addr <= *end);
        }

        true
    }

    pub fn is_aligned(&self, addr: u64) -> bool {
        addr % self.alignment as u64 == 0
    }
}

/// Result of validating a single pointer
#[derive(Debug, Clone)]
pub struct PointerValidationResult {
    /// The address being validated
    pub address: Address,
    /// The pointer value
    pub value: u64,
    /// Whether the pointer is valid
    pub is_valid: bool,
    /// Validation issues found
    pub issues: Vec<PointerIssue>,
    /// Target validation (if performed)
    pub target_result: Option<Box<PointerValidationResult>>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl PointerValidationResult {
    pub fn new(address: Address, value: u64) -> Self {
        Self {
            address,
            value,
            is_valid: true,
            issues: Vec::new(),
            target_result: None,
            confidence: 1.0,
        }
    }

    pub fn add_issue(&mut self, issue: PointerIssue) {
        self.confidence *= issue.severity_factor();
        if issue.is_error() {
            self.is_valid = false;
        }
        self.issues.push(issue);
    }

    pub fn with_target(mut self, target: PointerValidationResult) -> Self {
        if !target.is_valid {
            self.confidence *= 0.5;
        }
        self.target_result = Some(Box::new(target));
        self
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_error()).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_warning()).count()
    }
}

impl fmt::Display for PointerValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pointer @ {:016x} = {:016x}", self.address.as_u64(), self.value)?;
        writeln!(f, "  Valid: {}", self.is_valid)?;
        writeln!(f, "  Confidence: {:.2}%", self.confidence * 100.0)?;
        for issue in &self.issues {
            writeln!(f, "  Issue: {}", issue)?;
        }
        if let Some(ref target) = self.target_result {
            writeln!(f, "  Target:")?;
            writeln!(f, "    {}", target)?;
        }
        Ok(())
    }
}

/// Types of issues that can occur with pointers
#[derive(Debug, Clone)]
pub enum PointerIssue {
    /// Pointer is null
    NullPointer,
    /// Pointer is outside valid address range
    OutOfRange { value: u64, min: u64, max: u64 },
    /// Pointer has incorrect alignment
    Misaligned { value: u64, required: usize },
    /// Pointer target is not readable
    UnreadableTarget { value: u64 },
    /// Pointer points to unmapped memory
    UnmappedTarget { value: u64 },
    /// Pointer chain is too deep
    ChainTooDeep { depth: usize, max: usize },
    /// Pointer creates a cycle
    CyclicReference { addresses: Vec<u64> },
    /// Pointer target has unexpected content
    UnexpectedContent { expected: String, found: String },
    /// Generic warning
    Warning(String),
    /// Generic error
    Error(String),
}

impl PointerIssue {
    pub fn is_error(&self) -> bool {
        match self {
            PointerIssue::NullPointer => true,
            PointerIssue::OutOfRange { .. } => true,
            PointerIssue::UnmappedTarget { .. } => true,
            PointerIssue::Error(_) => true,
            _ => false,
        }
    }

    pub fn is_warning(&self) -> bool {
        !self.is_error()
    }

    pub fn severity_factor(&self) -> f64 {
        match self {
            PointerIssue::NullPointer => 0.0,
            PointerIssue::OutOfRange { .. } => 0.0,
            PointerIssue::Misaligned { .. } => 0.7,
            PointerIssue::UnreadableTarget { .. } => 0.3,
            PointerIssue::UnmappedTarget { .. } => 0.0,
            PointerIssue::ChainTooDeep { .. } => 0.8,
            PointerIssue::CyclicReference { .. } => 0.5,
            PointerIssue::UnexpectedContent { .. } => 0.6,
            PointerIssue::Warning(_) => 0.9,
            PointerIssue::Error(_) => 0.1,
        }
    }
}

impl fmt::Display for PointerIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PointerIssue::NullPointer => write!(f, "Null pointer"),
            PointerIssue::OutOfRange { value, min, max } => {
                write!(f, "Address 0x{:x} outside valid range [0x{:x}, 0x{:x}]", value, min, max)
            }
            PointerIssue::Misaligned { value, required } => {
                write!(f, "Address 0x{:x} not aligned to {} bytes", value, required)
            }
            PointerIssue::UnreadableTarget { value } => {
                write!(f, "Cannot read memory at 0x{:x}", value)
            }
            PointerIssue::UnmappedTarget { value } => {
                write!(f, "Address 0x{:x} is not mapped", value)
            }
            PointerIssue::ChainTooDeep { depth, max } => {
                write!(f, "Pointer chain depth {} exceeds maximum {}", depth, max)
            }
            PointerIssue::CyclicReference { addresses } => {
                let addrs: Vec<String> = addresses.iter().map(|a| format!("0x{:x}", a)).collect();
                write!(f, "Cyclic reference detected: {}", addrs.join(" -> "))
            }
            PointerIssue::UnexpectedContent { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            PointerIssue::Warning(msg) => write!(f, "Warning: {}", msg),
            PointerIssue::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Validates pointers in memory
pub struct PointerValidator {
    reader: Arc<dyn MemoryReader>,
    config: PointerValidationConfig,
    valid_regions: Vec<MemoryRegion>,
}

impl PointerValidator {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            config: PointerValidationConfig::default(),
            valid_regions: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: PointerValidationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_regions(mut self, regions: Vec<MemoryRegion>) -> Self {
        self.valid_regions = regions;
        self
    }

    /// Validate a single pointer at the given address
    pub fn validate_pointer(&self, address: Address) -> Result<PointerValidationResult, MemoryError> {
        let value = self.reader.read_u64(address)?;
        let mut result = PointerValidationResult::new(address, value);

        // Check for null
        if value == 0 {
            if !self.config.allow_null {
                result.add_issue(PointerIssue::NullPointer);
            }
            return Ok(result);
        }

        // Check range
        if !self.config.is_address_valid(value) {
            result.add_issue(PointerIssue::OutOfRange {
                value,
                min: self.config.min_address,
                max: self.config.max_address,
            });
            return Ok(result);
        }

        // Check alignment
        if !self.config.is_aligned(value) {
            result.add_issue(PointerIssue::Misaligned {
                value,
                required: self.config.alignment,
            });
        }

        // Validate target if requested
        if self.config.validate_targets {
            match self.reader.read_u64(Address::new(value)) {
                Ok(_) => {}
                Err(_) => {
                    result.add_issue(PointerIssue::UnreadableTarget { value });
                }
            }
        }

        Ok(result)
    }

    /// Validate a pointer chain (pointer to pointer to ...)
    pub fn validate_chain(&self, start: Address, expected_depth: usize) -> Result<Vec<PointerValidationResult>, MemoryError> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        let mut current = start;

        for depth in 0..expected_depth.min(self.config.max_chain_depth) {
            if visited.contains(&current.as_u64()) {
                let mut result = PointerValidationResult::new(current, 0);
                result.add_issue(PointerIssue::CyclicReference {
                    addresses: visited.iter().copied().collect(),
                });
                results.push(result);
                break;
            }

            visited.insert(current.as_u64());

            let result = self.validate_pointer(current)?;
            let next_addr = result.value;
            results.push(result);

            if next_addr == 0 || !self.config.is_address_valid(next_addr) {
                break;
            }

            current = Address::new(next_addr);
        }

        let results_len = results.len();
        if results_len > self.config.max_chain_depth {
            if let Some(last) = results.last_mut() {
                last.add_issue(PointerIssue::ChainTooDeep {
                    depth: results_len,
                    max: self.config.max_chain_depth,
                });
            }
        }

        Ok(results)
    }

    /// Validate an array of pointers
    pub fn validate_pointer_array(&self, base: Address, count: usize) -> Result<PointerArrayValidation, MemoryError> {
        let mut validation = PointerArrayValidation::new(base, count);

        for i in 0..count {
            let addr = base + (i * 8) as u64;
            let result = self.validate_pointer(addr)?;
            
            if result.is_valid {
                validation.valid_count += 1;
            } else {
                validation.invalid_indices.push(i);
            }
            
            validation.results.push(result);
        }

        validation.calculate_statistics();
        Ok(validation)
    }

    /// Check if an address looks like a valid vtable pointer
    pub fn validate_vtable_pointer(&self, address: Address) -> Result<VTablePointerValidation, MemoryError> {
        let vtable_addr = self.reader.read_u64(address)?;
        let mut validation = VTablePointerValidation::new(address, vtable_addr);

        // Basic pointer validation
        if vtable_addr == 0 {
            validation.add_issue("Null vtable pointer");
            return Ok(validation);
        }

        if !self.config.is_address_valid(vtable_addr) {
            validation.add_issue("VTable address out of range");
            return Ok(validation);
        }

        // Check vtable entries
        let mut valid_entries = 0;
        for i in 0..16 {
            let entry_addr = Address::new(vtable_addr + (i * 8) as u64);
            if let Ok(entry) = self.reader.read_u64(entry_addr) {
                if self.is_likely_function_pointer(entry) {
                    valid_entries += 1;
                }
            } else {
                break;
            }
        }

        validation.function_count = valid_entries;
        validation.is_valid_vtable = valid_entries >= 3;
        validation.confidence = (valid_entries as f64 / 16.0).min(1.0);

        Ok(validation)
    }

    fn is_likely_function_pointer(&self, addr: u64) -> bool {
        if addr < self.config.min_address || addr > self.config.max_address {
            return false;
        }

        // Must be 4-byte aligned for ARM64
        if addr % 4 != 0 {
            return false;
        }

        // Try to read first instruction
        if let Ok(bytes) = self.reader.read_bytes(Address::new(addr), 4) {
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            // Not all zeros or all ones
            insn != 0 && insn != 0xFFFFFFFF
        } else {
            false
        }
    }

    /// Validate a structure pointer (checks fields)
    pub fn validate_structure_pointer(&self, address: Address, field_offsets: &[(usize, PointerExpectation)]) -> Result<StructurePointerValidation, MemoryError> {
        let base = self.reader.read_u64(address)?;
        let mut validation = StructurePointerValidation::new(address, base);

        if base == 0 || !self.config.is_address_valid(base) {
            validation.add_issue("Invalid base pointer");
            return Ok(validation);
        }

        for (offset, expectation) in field_offsets {
            let field_addr = Address::new(base + *offset as u64);
            let field_value = self.reader.read_u64(field_addr)?;

            let is_valid = match expectation {
                PointerExpectation::ValidPointer => self.config.is_address_valid(field_value),
                PointerExpectation::NullOrValid => field_value == 0 || self.config.is_address_valid(field_value),
                PointerExpectation::NonZero => field_value != 0,
                PointerExpectation::Zero => field_value == 0,
                PointerExpectation::InRange(min, max) => field_value >= *min && field_value <= *max,
                PointerExpectation::Any => true,
            };

            validation.field_results.push((*offset, field_value, is_valid));
            if is_valid {
                validation.valid_fields += 1;
            }
        }

        validation.total_fields = field_offsets.len();
        validation.confidence = if validation.total_fields > 0 {
            validation.valid_fields as f64 / validation.total_fields as f64
        } else {
            1.0
        };

        Ok(validation)
    }
}

/// Expected value for a pointer field
#[derive(Debug, Clone)]
pub enum PointerExpectation {
    /// Must be a valid pointer
    ValidPointer,
    /// Can be null or a valid pointer
    NullOrValid,
    /// Must be non-zero
    NonZero,
    /// Must be zero
    Zero,
    /// Must be in specified range
    InRange(u64, u64),
    /// Any value is acceptable
    Any,
}

/// Validation result for a pointer array
#[derive(Debug, Clone)]
pub struct PointerArrayValidation {
    pub base: Address,
    pub count: usize,
    pub results: Vec<PointerValidationResult>,
    pub valid_count: usize,
    pub invalid_indices: Vec<usize>,
    pub null_count: usize,
    pub unique_targets: usize,
}

impl PointerArrayValidation {
    pub fn new(base: Address, count: usize) -> Self {
        Self {
            base,
            count,
            results: Vec::new(),
            valid_count: 0,
            invalid_indices: Vec::new(),
            null_count: 0,
            unique_targets: 0,
        }
    }

    fn calculate_statistics(&mut self) {
        self.null_count = self.results.iter().filter(|r| r.value == 0).count();
        
        let targets: HashSet<u64> = self.results.iter()
            .filter(|r| r.value != 0)
            .map(|r| r.value)
            .collect();
        self.unique_targets = targets.len();
    }

    pub fn validity_percentage(&self) -> f64 {
        if self.count == 0 {
            return 100.0;
        }
        (self.valid_count as f64 / self.count as f64) * 100.0
    }

    pub fn is_likely_vtable(&self) -> bool {
        self.valid_count >= 3 && 
        self.validity_percentage() > 50.0 &&
        self.null_count < self.count / 2
    }
}

impl fmt::Display for PointerArrayValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pointer Array @ {:016x}", self.base.as_u64())?;
        writeln!(f, "  Count: {}", self.count)?;
        writeln!(f, "  Valid: {} ({:.1}%)", self.valid_count, self.validity_percentage())?;
        writeln!(f, "  Null: {}", self.null_count)?;
        writeln!(f, "  Unique targets: {}", self.unique_targets)?;
        writeln!(f, "  Likely vtable: {}", self.is_likely_vtable())?;
        Ok(())
    }
}

/// Validation result for a vtable pointer
#[derive(Debug, Clone)]
pub struct VTablePointerValidation {
    pub address: Address,
    pub vtable_address: u64,
    pub is_valid_vtable: bool,
    pub function_count: usize,
    pub confidence: f64,
    pub issues: Vec<String>,
}

impl VTablePointerValidation {
    pub fn new(address: Address, vtable_address: u64) -> Self {
        Self {
            address,
            vtable_address,
            is_valid_vtable: false,
            function_count: 0,
            confidence: 0.0,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: &str) {
        self.issues.push(issue.to_string());
        self.is_valid_vtable = false;
    }
}

impl fmt::Display for VTablePointerValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VTable Pointer @ {:016x}", self.address.as_u64())?;
        writeln!(f, "  VTable address: {:016x}", self.vtable_address)?;
        writeln!(f, "  Valid: {}", self.is_valid_vtable)?;
        writeln!(f, "  Function count: {}", self.function_count)?;
        writeln!(f, "  Confidence: {:.1}%", self.confidence * 100.0)?;
        for issue in &self.issues {
            writeln!(f, "  Issue: {}", issue)?;
        }
        Ok(())
    }
}

/// Validation result for a structure pointer
#[derive(Debug, Clone)]
pub struct StructurePointerValidation {
    pub address: Address,
    pub base: u64,
    pub field_results: Vec<(usize, u64, bool)>, // (offset, value, is_valid)
    pub valid_fields: usize,
    pub total_fields: usize,
    pub confidence: f64,
    pub issues: Vec<String>,
}

impl StructurePointerValidation {
    pub fn new(address: Address, base: u64) -> Self {
        Self {
            address,
            base,
            field_results: Vec::new(),
            valid_fields: 0,
            total_fields: 0,
            confidence: 0.0,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: &str) {
        self.issues.push(issue.to_string());
    }

    pub fn is_valid(&self) -> bool {
        self.issues.is_empty() && self.valid_fields == self.total_fields
    }
}

impl fmt::Display for StructurePointerValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Structure Pointer @ {:016x}", self.address.as_u64())?;
        writeln!(f, "  Base: {:016x}", self.base)?;
        writeln!(f, "  Valid fields: {}/{}", self.valid_fields, self.total_fields)?;
        writeln!(f, "  Confidence: {:.1}%", self.confidence * 100.0)?;
        for (offset, value, valid) in &self.field_results {
            let status = if *valid { "OK" } else { "INVALID" };
            writeln!(f, "  +0x{:x}: {:016x} [{}]", offset, value, status)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = PointerValidationConfig::default();
        assert!(config.is_address_valid(0x200000000));
        assert!(!config.is_address_valid(0));
        assert!(!config.is_address_valid(0x1000));
    }

    #[test]
    fn test_config_alignment() {
        let config = PointerValidationConfig::default();
        assert!(config.is_aligned(0x200000000));
        assert!(config.is_aligned(0x200000008));
        assert!(!config.is_aligned(0x200000001));
    }

    #[test]
    fn test_pointer_issue_severity() {
        assert!(PointerIssue::NullPointer.is_error());
        assert!(PointerIssue::Misaligned { value: 0, required: 8 }.is_warning());
    }
}
