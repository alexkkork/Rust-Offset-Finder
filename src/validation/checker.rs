// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disassembler::Disassembler;
use crate::analysis::function::AnalyzedFunction;
use crate::validation::validator::ValidationIssue;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

pub struct OffsetChecker {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
}

impl OffsetChecker {
    pub fn new(reader: Arc<dyn MemoryReader>, disassembler: Arc<Disassembler>) -> Self {
        Self { reader, disassembler }
    }

    pub fn check_function(&self, addr: Address) -> CheckResult {
        let mut checks = Vec::new();

        checks.push(self.check_address_valid(addr));
        checks.push(self.check_alignment(addr, 4));
        checks.push(self.check_in_executable_region(addr));
        checks.push(self.check_function_prologue(addr));

        CheckResult::from_checks(checks)
    }

    pub fn check_structure_offset(&self, base: Address, offset: u64, expected_type: OffsetType) -> CheckResult {
        let mut checks = Vec::new();
        let target = base + offset;

        checks.push(self.check_address_valid(target));

        let alignment = match expected_type {
            OffsetType::Pointer => 8,
            OffsetType::Integer64 => 8,
            OffsetType::Integer32 => 4,
            OffsetType::Integer16 => 2,
            OffsetType::Byte => 1,
            OffsetType::Float64 => 8,
            OffsetType::Float32 => 4,
            _ => 1,
        };
        checks.push(self.check_offset_alignment(offset, alignment));

        if matches!(expected_type, OffsetType::Pointer) {
            checks.push(self.check_pointer_valid(target));
        }

        CheckResult::from_checks(checks)
    }

    pub fn check_vtable(&self, addr: Address) -> CheckResult {
        let mut checks = Vec::new();

        checks.push(self.check_address_valid(addr));
        checks.push(self.check_alignment(addr, 8));

        let vtable_check = self.check_vtable_entries(addr);
        checks.push(vtable_check);

        CheckResult::from_checks(checks)
    }

    pub fn cross_validate_offsets(&self, offsets: &HashMap<String, u64>) -> CrossValidationResult {
        let mut issues = Vec::new();
        let mut conflicts = Vec::new();

        let values: HashSet<u64> = offsets.values().cloned().collect();
        if values.len() != offsets.len() {
            let mut seen: HashMap<u64, Vec<String>> = HashMap::new();
            for (name, &value) in offsets {
                seen.entry(value).or_default().push(name.clone());
            }

            for (value, names) in seen {
                if names.len() > 1 {
                    conflicts.push(OffsetConflict {
                        offset_value: value,
                        conflicting_names: names,
                    });
                }
            }
        }

        let sorted_offsets: Vec<_> = {
            let mut v: Vec<_> = offsets.iter().collect();
            v.sort_by_key(|(_, &offset)| offset);
            v
        };

        for window in sorted_offsets.windows(2) {
            let (name1, &offset1) = window[0];
            let (name2, &offset2) = window[1];

            if offset2 - offset1 < 4 && offset2 != offset1 {
                issues.push(format!("Offsets '{}' (0x{:x}) and '{}' (0x{:x}) may overlap",
                    name1, offset1, name2, offset2));
            }
        }

        CrossValidationResult {
            valid: conflicts.is_empty() && issues.is_empty(),
            conflicts,
            issues,
        }
    }

    pub fn verify_function_behavior(&self, addr: Address, expected: &ExpectedBehavior) -> BehaviorCheckResult {
        let mut matches = Vec::new();
        let mut mismatches = Vec::new();

        if let Ok(instrs) = self.disassembler.disassemble_function(addr, 256) {
            if let Some(expected_call) = &expected.must_call {
                let has_call = instrs.iter()
                    .filter(|i| i.is_call())
                    .any(|i| i.op_str.contains(expected_call));

                if has_call {
                    matches.push(format!("Calls {}", expected_call));
                } else {
                    mismatches.push(format!("Expected call to {} not found", expected_call));
                }
            }

            if let Some(min_size) = expected.minimum_size {
                let actual_size = instrs.len() * 4;
                if actual_size >= min_size {
                    matches.push(format!("Size {} >= minimum {}", actual_size, min_size));
                } else {
                    mismatches.push(format!("Size {} < minimum {}", actual_size, min_size));
                }
            }

            if let Some(max_size) = expected.maximum_size {
                let actual_size = instrs.len() * 4;
                if actual_size <= max_size {
                    matches.push(format!("Size {} <= maximum {}", actual_size, max_size));
                } else {
                    mismatches.push(format!("Size {} > maximum {}", actual_size, max_size));
                }
            }

            if expected.must_have_prologue {
                if let Some(first) = instrs.first() {
                    if first.mnemonic.starts_with("STP") && first.op_str.contains("X29") {
                        matches.push("Has standard prologue".to_string());
                    } else {
                        mismatches.push("Missing standard prologue".to_string());
                    }
                }
            }
        }

        BehaviorCheckResult {
            matches_expected: mismatches.is_empty(),
            matches,
            mismatches,
        }
    }

    fn check_address_valid(&self, addr: Address) -> SingleCheck {
        if addr.as_u64() == 0 {
            return SingleCheck::failed("Address is null");
        }

        if let Ok(regions) = self.reader.get_regions() {
            let in_region = regions.iter()
                .any(|r| addr.as_u64() >= r.range.start.as_u64() &&
                         addr.as_u64() < r.range.end.as_u64());

            if !in_region {
                return SingleCheck::failed("Address not in valid memory region");
            }
        }

        SingleCheck::passed("Address is valid")
    }

    fn check_alignment(&self, addr: Address, alignment: u64) -> SingleCheck {
        if addr.as_u64() % alignment == 0 {
            SingleCheck::passed(&format!("Address is {}-byte aligned", alignment))
        } else {
            SingleCheck::warning(&format!("Address is not {}-byte aligned", alignment))
        }
    }

    fn check_offset_alignment(&self, offset: u64, alignment: u64) -> SingleCheck {
        if offset % alignment == 0 {
            SingleCheck::passed(&format!("Offset is {}-byte aligned", alignment))
        } else {
            SingleCheck::warning(&format!("Offset is not {}-byte aligned", alignment))
        }
    }

    fn check_in_executable_region(&self, addr: Address) -> SingleCheck {
        if let Ok(regions) = self.reader.get_regions() {
            let in_exec = regions.iter()
                .any(|r| r.protection.is_executable() &&
                         addr.as_u64() >= r.range.start.as_u64() &&
                         addr.as_u64() < r.range.end.as_u64());

            if in_exec {
                return SingleCheck::passed("Address is in executable region");
            } else {
                return SingleCheck::failed("Address not in executable region");
            }
        }

        SingleCheck::warning("Could not verify executable region")
    }

    fn check_function_prologue(&self, addr: Address) -> SingleCheck {
        if let Ok(bytes) = self.reader.read_bytes(addr, 8) {
            let first_word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            let is_stp = (first_word & 0xFE000000) == 0xA9000000;
            let is_sub_sp = (first_word & 0xFF0003FF) == 0xD10003FF;

            if is_stp || is_sub_sp {
                return SingleCheck::passed("Valid function prologue detected");
            }

            return SingleCheck::warning("Non-standard function prologue");
        }

        SingleCheck::failed("Could not read function prologue")
    }

    fn check_pointer_valid(&self, addr: Address) -> SingleCheck {
        if let Ok(ptr_value) = self.reader.read_u64(addr) {
            if ptr_value == 0 {
                return SingleCheck::passed("Pointer is null (may be valid)");
            }

            if ptr_value % 8 != 0 {
                return SingleCheck::warning("Pointer value is not 8-byte aligned");
            }

            if let Ok(regions) = self.reader.get_regions() {
                let valid = regions.iter()
                    .any(|r| ptr_value >= r.range.start.as_u64() &&
                             ptr_value < r.range.end.as_u64());

                if valid {
                    return SingleCheck::passed("Pointer points to valid memory");
                } else {
                    return SingleCheck::failed("Pointer points to invalid memory");
                }
            }
        }

        SingleCheck::failed("Could not read pointer value")
    }

    fn check_vtable_entries(&self, addr: Address) -> SingleCheck {
        if let Ok(bytes) = self.reader.read_bytes(addr, 64) {
            let mut valid_entries = 0;

            for i in 0..8 {
                let entry = u64::from_le_bytes(
                    bytes[i*8..(i+1)*8].try_into().unwrap_or([0; 8])
                );

                if entry != 0 && entry % 4 == 0 && entry > 0x100000000 {
                    valid_entries += 1;
                }
            }

            if valid_entries >= 3 {
                return SingleCheck::passed(&format!("{}/8 vtable entries look valid", valid_entries));
            } else if valid_entries >= 1 {
                return SingleCheck::warning(&format!("Only {}/8 vtable entries look valid", valid_entries));
            } else {
                return SingleCheck::failed("No valid vtable entries found");
            }
        }

        SingleCheck::failed("Could not read vtable entries")
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub passed: bool,
    pub checks: Vec<SingleCheck>,
    pub confidence: f64,
}

impl CheckResult {
    pub fn from_checks(checks: Vec<SingleCheck>) -> Self {
        let passed_count = checks.iter().filter(|c| c.status == CheckStatus::Passed).count();
        let warning_count = checks.iter().filter(|c| c.status == CheckStatus::Warning).count();
        let failed_count = checks.iter().filter(|c| c.status == CheckStatus::Failed).count();

        let total = checks.len() as f64;
        let confidence = if total > 0.0 {
            (passed_count as f64 + warning_count as f64 * 0.5) / total
        } else {
            0.0
        };

        let passed = failed_count == 0;

        Self {
            passed,
            checks,
            confidence,
        }
    }

    pub fn passed_checks(&self) -> Vec<&SingleCheck> {
        self.checks.iter().filter(|c| c.status == CheckStatus::Passed).collect()
    }

    pub fn failed_checks(&self) -> Vec<&SingleCheck> {
        self.checks.iter().filter(|c| c.status == CheckStatus::Failed).collect()
    }

    pub fn warning_checks(&self) -> Vec<&SingleCheck> {
        self.checks.iter().filter(|c| c.status == CheckStatus::Warning).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SingleCheck {
    pub status: CheckStatus,
    pub message: String,
}

impl SingleCheck {
    pub fn passed(message: &str) -> Self {
        Self {
            status: CheckStatus::Passed,
            message: message.to_string(),
        }
    }

    pub fn warning(message: &str) -> Self {
        Self {
            status: CheckStatus::Warning,
            message: message.to_string(),
        }
    }

    pub fn failed(message: &str) -> Self {
        Self {
            status: CheckStatus::Failed,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetType {
    Pointer,
    Integer64,
    Integer32,
    Integer16,
    Byte,
    Float64,
    Float32,
    Array,
    Struct,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CrossValidationResult {
    pub valid: bool,
    pub conflicts: Vec<OffsetConflict>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OffsetConflict {
    pub offset_value: u64,
    pub conflicting_names: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExpectedBehavior {
    pub must_call: Option<String>,
    pub minimum_size: Option<usize>,
    pub maximum_size: Option<usize>,
    pub must_have_prologue: bool,
}

impl ExpectedBehavior {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn must_call(mut self, func: &str) -> Self {
        self.must_call = Some(func.to_string());
        self
    }

    pub fn min_size(mut self, size: usize) -> Self {
        self.minimum_size = Some(size);
        self
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.maximum_size = Some(size);
        self
    }

    pub fn with_prologue(mut self) -> Self {
        self.must_have_prologue = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorCheckResult {
    pub matches_expected: bool,
    pub matches: Vec<String>,
    pub mismatches: Vec<String>,
}
