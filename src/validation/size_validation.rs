// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Size and boundary validation for offsets
pub struct SizeValidator {
    reader: Arc<dyn MemoryReader>,
    known_sizes: HashMap<String, ExpectedSize>,
}

impl SizeValidator {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let mut known_sizes = HashMap::new();
        
        // Common Luau structure sizes (ARM64)
        known_sizes.insert("lua_State".to_string(), ExpectedSize::range(0x100, 0x400));
        known_sizes.insert("global_State".to_string(), ExpectedSize::range(0x200, 0x800));
        known_sizes.insert("TValue".to_string(), ExpectedSize::exact(16));
        known_sizes.insert("TString".to_string(), ExpectedSize::minimum(32));
        known_sizes.insert("Table".to_string(), ExpectedSize::range(0x30, 0x60));
        known_sizes.insert("Closure".to_string(), ExpectedSize::range(0x30, 0x100));
        known_sizes.insert("Proto".to_string(), ExpectedSize::range(0x50, 0x200));
        known_sizes.insert("Udata".to_string(), ExpectedSize::minimum(24));
        known_sizes.insert("GCHeader".to_string(), ExpectedSize::exact(8));
        known_sizes.insert("CallInfo".to_string(), ExpectedSize::range(0x20, 0x40));

        Self {
            reader,
            known_sizes,
        }
    }

    pub fn add_known_size(&mut self, name: &str, expected: ExpectedSize) {
        self.known_sizes.insert(name.to_string(), expected);
    }

    /// Validate an offset falls within expected structure bounds
    pub fn validate_offset(&self, structure: &str, offset: usize) -> SizeValidationResult {
        let mut result = SizeValidationResult::new(structure, offset);

        if let Some(expected) = self.known_sizes.get(structure) {
            result.expected = Some(expected.clone());
            result.is_valid = expected.is_valid_offset(offset);
            
            if !result.is_valid {
                result.issues.push(format!(
                    "Offset 0x{:x} outside expected {} bounds ({})",
                    offset, structure, expected
                ));
            }
        } else {
            result.issues.push(format!("No size information for {}", structure));
        }

        result
    }

    /// Validate total structure size
    pub fn validate_structure_size(&self, structure: &str, actual_size: usize) -> SizeValidationResult {
        let mut result = SizeValidationResult::new(structure, actual_size);

        if let Some(expected) = self.known_sizes.get(structure) {
            result.expected = Some(expected.clone());
            result.is_valid = expected.is_valid_size(actual_size);
            
            if !result.is_valid {
                result.issues.push(format!(
                    "Structure size 0x{:x} doesn't match expected {} ({})",
                    actual_size, structure, expected
                ));
            }
        }

        result
    }

    /// Infer structure size from offsets
    pub fn infer_size(&self, offsets: &[usize]) -> InferredSize {
        if offsets.is_empty() {
            return InferredSize {
                minimum: 0,
                maximum: 0,
                gaps: Vec::new(),
                alignment: 1,
            };
        }

        let mut sorted = offsets.to_vec();
        sorted.sort();

        let minimum = *sorted.last().unwrap() + 8; // At least last offset + 8
        let maximum = minimum + 0x100; // Allow for padding

        // Find gaps
        let mut gaps = Vec::new();
        for window in sorted.windows(2) {
            let gap = window[1] - window[0];
            if gap > 8 {
                gaps.push((window[0], gap));
            }
        }

        // Infer alignment
        let alignment = sorted.iter()
            .map(|&o| if o == 0 { 8 } else { gcd(o, 8) })
            .min()
            .unwrap_or(1);

        InferredSize {
            minimum,
            maximum,
            gaps,
            alignment,
        }
    }

    /// Validate field alignment
    pub fn validate_alignment(&self, offset: usize, size: usize) -> AlignmentValidation {
        let mut result = AlignmentValidation::new(offset, size);

        // Determine required alignment based on size
        let required_alignment = match size {
            1 => 1,
            2 => 2,
            4 => 4,
            _ => 8,
        };

        result.required_alignment = required_alignment;
        result.is_aligned = offset % required_alignment == 0;

        if !result.is_aligned {
            result.padding_needed = required_alignment - (offset % required_alignment);
        }

        result
    }

    /// Calculate total size including padding
    pub fn calculate_padded_size(&self, fields: &[(usize, usize)]) -> PaddedSizeCalculation {
        let mut calc = PaddedSizeCalculation::new();

        for &(offset, size) in fields {
            let alignment = self.validate_alignment(offset, size);
            if !alignment.is_aligned {
                calc.add_padding(offset, alignment.padding_needed);
            }
            calc.add_field(offset, size);
        }

        calc.finalize();
        calc
    }

    /// Validate boundary crossing
    pub fn validate_boundaries(&self, base: u64, size: usize) -> BoundaryValidation {
        let mut result = BoundaryValidation::new(base, size);

        let end = base + size as u64;

        // Check page boundary crossing
        let start_page = base / 0x1000;
        let end_page = end / 0x1000;
        result.crosses_page = start_page != end_page;

        // Check cache line crossing (64 bytes typically)
        let start_cache_line = base / 64;
        let end_cache_line = end / 64;
        result.crosses_cache_line = start_cache_line != end_cache_line;

        // Check if extends beyond typical stack frame
        result.exceeds_stack_frame = size > 0x10000;

        result
    }

    /// Validate array bounds
    pub fn validate_array(&self, base: Address, element_size: usize, count: usize) -> ArrayValidation {
        let mut result = ArrayValidation::new(base, element_size, count);

        result.total_size = element_size * count;
        
        // Check if size is reasonable
        result.is_reasonable = result.total_size < 0x1000000; // 16 MB limit

        // Check alignment
        result.elements_aligned = element_size % 8 == 0 || element_size <= 8;

        // Calculate end address
        result.end_address = base + result.total_size as u64;

        result
    }
}

/// Expected size specification
#[derive(Debug, Clone)]
pub enum ExpectedSize {
    /// Exact size expected
    Exact(usize),
    /// Size must be at least this
    Minimum(usize),
    /// Size must be at most this
    Maximum(usize),
    /// Size must be in range
    Range(usize, usize),
}

impl ExpectedSize {
    pub fn exact(size: usize) -> Self {
        ExpectedSize::Exact(size)
    }

    pub fn minimum(size: usize) -> Self {
        ExpectedSize::Minimum(size)
    }

    pub fn maximum(size: usize) -> Self {
        ExpectedSize::Maximum(size)
    }

    pub fn range(min: usize, max: usize) -> Self {
        ExpectedSize::Range(min, max)
    }

    pub fn is_valid_size(&self, size: usize) -> bool {
        match self {
            ExpectedSize::Exact(expected) => size == *expected,
            ExpectedSize::Minimum(min) => size >= *min,
            ExpectedSize::Maximum(max) => size <= *max,
            ExpectedSize::Range(min, max) => size >= *min && size <= *max,
        }
    }

    pub fn is_valid_offset(&self, offset: usize) -> bool {
        match self {
            ExpectedSize::Exact(expected) => offset < *expected,
            ExpectedSize::Minimum(_) => true,
            ExpectedSize::Maximum(max) => offset < *max,
            ExpectedSize::Range(_, max) => offset < *max,
        }
    }
}

impl fmt::Display for ExpectedSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedSize::Exact(s) => write!(f, "exactly 0x{:x}", s),
            ExpectedSize::Minimum(s) => write!(f, "at least 0x{:x}", s),
            ExpectedSize::Maximum(s) => write!(f, "at most 0x{:x}", s),
            ExpectedSize::Range(min, max) => write!(f, "0x{:x} - 0x{:x}", min, max),
        }
    }
}

/// Result of size validation
#[derive(Debug, Clone)]
pub struct SizeValidationResult {
    pub structure: String,
    pub value: usize,
    pub expected: Option<ExpectedSize>,
    pub is_valid: bool,
    pub issues: Vec<String>,
}

impl SizeValidationResult {
    pub fn new(structure: &str, value: usize) -> Self {
        Self {
            structure: structure.to_string(),
            value,
            expected: None,
            is_valid: true,
            issues: Vec::new(),
        }
    }
}

impl fmt::Display for SizeValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: 0x{:x}", self.structure, self.value)?;
        if let Some(ref expected) = self.expected {
            write!(f, " (expected: {})", expected)?;
        }
        write!(f, " - {}", if self.is_valid { "VALID" } else { "INVALID" })
    }
}

/// Inferred size from offsets
#[derive(Debug, Clone)]
pub struct InferredSize {
    pub minimum: usize,
    pub maximum: usize,
    pub gaps: Vec<(usize, usize)>, // (offset, gap_size)
    pub alignment: usize,
}

impl fmt::Display for InferredSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inferred size: 0x{:x} - 0x{:x}, alignment: {}, gaps: {}",
            self.minimum, self.maximum, self.alignment, self.gaps.len())
    }
}

/// Alignment validation result
#[derive(Debug, Clone)]
pub struct AlignmentValidation {
    pub offset: usize,
    pub size: usize,
    pub required_alignment: usize,
    pub is_aligned: bool,
    pub padding_needed: usize,
}

impl AlignmentValidation {
    pub fn new(offset: usize, size: usize) -> Self {
        Self {
            offset,
            size,
            required_alignment: 1,
            is_aligned: true,
            padding_needed: 0,
        }
    }
}

impl fmt::Display for AlignmentValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Offset 0x{:x}, size {}, alignment {}: ",
            self.offset, self.size, self.required_alignment)?;
        if self.is_aligned {
            write!(f, "aligned")
        } else {
            write!(f, "needs {} bytes padding", self.padding_needed)
        }
    }
}

/// Padded size calculation
#[derive(Debug, Clone)]
pub struct PaddedSizeCalculation {
    pub fields: Vec<(usize, usize)>,
    pub padding: Vec<(usize, usize)>,
    pub raw_size: usize,
    pub padded_size: usize,
    pub total_padding: usize,
}

impl PaddedSizeCalculation {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            padding: Vec::new(),
            raw_size: 0,
            padded_size: 0,
            total_padding: 0,
        }
    }

    pub fn add_field(&mut self, offset: usize, size: usize) {
        self.fields.push((offset, size));
    }

    pub fn add_padding(&mut self, offset: usize, amount: usize) {
        self.padding.push((offset, amount));
        self.total_padding += amount;
    }

    pub fn finalize(&mut self) {
        if let Some(&(last_offset, last_size)) = self.fields.last() {
            self.raw_size = last_offset + last_size;
        }
        self.padded_size = self.raw_size + self.total_padding;
        
        // Align to 8 bytes
        self.padded_size = (self.padded_size + 7) & !7;
    }
}

impl Default for PaddedSizeCalculation {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PaddedSizeCalculation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Raw: 0x{:x}, Padded: 0x{:x}, Padding: {} bytes ({:.1}%)",
            self.raw_size, self.padded_size, self.total_padding,
            if self.padded_size > 0 { self.total_padding as f64 / self.padded_size as f64 * 100.0 } else { 0.0 })
    }
}

/// Boundary validation result
#[derive(Debug, Clone)]
pub struct BoundaryValidation {
    pub base: u64,
    pub size: usize,
    pub end: u64,
    pub crosses_page: bool,
    pub crosses_cache_line: bool,
    pub exceeds_stack_frame: bool,
}

impl BoundaryValidation {
    pub fn new(base: u64, size: usize) -> Self {
        Self {
            base,
            size,
            end: base + size as u64,
            crosses_page: false,
            crosses_cache_line: false,
            exceeds_stack_frame: false,
        }
    }

    pub fn has_issues(&self) -> bool {
        self.exceeds_stack_frame
    }
}

impl fmt::Display for BoundaryValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x} - 0x{:x} (size 0x{:x})", self.base, self.end, self.size)?;
        if self.crosses_page {
            write!(f, " [crosses page]")?;
        }
        if self.crosses_cache_line {
            write!(f, " [crosses cache line]")?;
        }
        if self.exceeds_stack_frame {
            write!(f, " [exceeds stack frame]")?;
        }
        Ok(())
    }
}

/// Array validation result
#[derive(Debug, Clone)]
pub struct ArrayValidation {
    pub base: Address,
    pub element_size: usize,
    pub count: usize,
    pub total_size: usize,
    pub end_address: Address,
    pub is_reasonable: bool,
    pub elements_aligned: bool,
}

impl ArrayValidation {
    pub fn new(base: Address, element_size: usize, count: usize) -> Self {
        Self {
            base,
            element_size,
            count,
            total_size: 0,
            end_address: base,
            is_reasonable: true,
            elements_aligned: true,
        }
    }

    pub fn get_element_address(&self, index: usize) -> Option<Address> {
        if index >= self.count {
            return None;
        }
        Some(self.base + (index * self.element_size) as u64)
    }
}

impl fmt::Display for ArrayValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Array[{}] of {} bytes at {:016x}, total 0x{:x}",
            self.count, self.element_size, self.base.as_u64(), self.total_size)?;
        if !self.is_reasonable {
            write!(f, " [size unreasonable]")?;
        }
        if !self.elements_aligned {
            write!(f, " [misaligned elements]")?;
        }
        Ok(())
    }
}

/// GCD helper
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Size comparison utility
pub struct SizeComparator;

impl SizeComparator {
    /// Compare sizes between versions
    pub fn compare(old_size: usize, new_size: usize) -> SizeComparison {
        SizeComparison {
            old_size,
            new_size,
            difference: new_size as i64 - old_size as i64,
            percentage_change: if old_size > 0 {
                ((new_size as f64 - old_size as f64) / old_size as f64) * 100.0
            } else {
                0.0
            },
            grew: new_size > old_size,
            shrunk: new_size < old_size,
            same: new_size == old_size,
        }
    }
}

/// Result of size comparison
#[derive(Debug, Clone)]
pub struct SizeComparison {
    pub old_size: usize,
    pub new_size: usize,
    pub difference: i64,
    pub percentage_change: f64,
    pub grew: bool,
    pub shrunk: bool,
    pub same: bool,
}

impl fmt::Display for SizeComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.same {
            write!(f, "0x{:x} (unchanged)", self.old_size)
        } else {
            let sign = if self.grew { "+" } else { "" };
            write!(f, "0x{:x} -> 0x{:x} ({}{} bytes, {:.1}%)",
                self.old_size, self.new_size, sign, self.difference, self.percentage_change)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_size() {
        let exact = ExpectedSize::exact(16);
        assert!(exact.is_valid_size(16));
        assert!(!exact.is_valid_size(20));

        let range = ExpectedSize::range(10, 20);
        assert!(range.is_valid_size(15));
        assert!(!range.is_valid_size(25));
    }

    #[test]
    fn test_alignment_validation() {
        // 4-byte field at offset 0 - aligned
        let aligned = AlignmentValidation { offset: 0, size: 4, required_alignment: 4, is_aligned: true, padding_needed: 0 };
        assert!(aligned.is_aligned);

        // 8-byte field at offset 4 - misaligned
        let misaligned = AlignmentValidation { offset: 4, size: 8, required_alignment: 8, is_aligned: false, padding_needed: 4 };
        assert!(!misaligned.is_aligned);
    }

    #[test]
    fn test_size_comparison() {
        let result = SizeComparator::compare(100, 150);
        assert!(result.grew);
        assert_eq!(result.difference, 50);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(8, 8), 8);
        assert_eq!(gcd(16, 8), 8);
    }
}
