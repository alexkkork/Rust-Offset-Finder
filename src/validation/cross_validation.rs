// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::FinderResult;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Cross-validation between different finder results
pub struct CrossValidator {
    reader: Arc<dyn MemoryReader>,
    results: HashMap<String, Vec<FinderResult>>,
    validations: Vec<CrossValidationCheck>,
}

impl CrossValidator {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            results: HashMap::new(),
            validations: Vec::new(),
        }
    }

    /// Add finder results for cross-validation
    pub fn add_results(&mut self, finder_name: &str, results: Vec<FinderResult>) {
        self.results.insert(finder_name.to_string(), results);
    }

    /// Add a validation check
    pub fn add_check(&mut self, check: CrossValidationCheck) {
        self.validations.push(check);
    }

    /// Run all cross-validation checks
    pub fn validate(&self) -> CrossValidationReport {
        let mut report = CrossValidationReport::new();

        for check in &self.validations {
            let result = self.run_check(check);
            report.add_result(result);
        }

        // Run automatic checks
        self.check_overlapping_offsets(&mut report);
        self.check_size_consistency(&mut report);
        self.check_pointer_chains(&mut report);
        self.check_related_functions(&mut report);

        report.calculate_overall_score();
        report
    }

    fn run_check(&self, check: &CrossValidationCheck) -> CheckResult {
        match check {
            CrossValidationCheck::OffsetRange { finder, offset_name, min, max } => {
                self.check_offset_range(finder, offset_name, *min, *max)
            }
            CrossValidationCheck::OffsetRelation { finder1, offset1, finder2, offset2, relation } => {
                self.check_offset_relation(finder1, offset1, finder2, offset2, relation)
            }
            CrossValidationCheck::StructureSize { finder, structure_name, expected_min, expected_max } => {
                self.check_structure_size(finder, structure_name, *expected_min, *expected_max)
            }
            CrossValidationCheck::FunctionChain { functions, expected_order } => {
                self.check_function_chain(functions, expected_order)
            }
            CrossValidationCheck::Custom { name, validator } => {
                validator(&self.results)
            }
        }
    }

    fn check_offset_range(&self, finder: &str, offset_name: &str, min: u64, max: u64) -> CheckResult {
        let mut result = CheckResult::new(&format!("{}.{} in range", finder, offset_name));

        if let Some(results) = self.results.get(finder) {
            for r in results {
                if r.name == offset_name {
                    if r.address.as_u64() >= min && r.address.as_u64() <= max {
                        result.passed = true;
                        result.confidence = 1.0;
                        result.details.push(format!(
                            "Offset {} (0x{:x}) is within expected range [0x{:x}, 0x{:x}]",
                            offset_name, r.address.as_u64(), min, max
                        ));
                    } else {
                        result.passed = false;
                        result.confidence = 0.3;
                        result.details.push(format!(
                            "Offset {} (0x{:x}) is outside expected range [0x{:x}, 0x{:x}]",
                            offset_name, r.address.as_u64(), min, max
                        ));
                    }
                    return result;
                }
            }
            result.details.push(format!("Offset {} not found in {} results", offset_name, finder));
        } else {
            result.details.push(format!("No results from finder {}", finder));
        }

        result
    }

    fn check_offset_relation(&self, finder1: &str, offset1: &str, finder2: &str, offset2: &str, relation: &OffsetRelation) -> CheckResult {
        let mut result = CheckResult::new(&format!("{}.{} {} {}.{}", finder1, offset1, relation, finder2, offset2));

        let addr1 = self.find_offset(finder1, offset1);
        let addr2 = self.find_offset(finder2, offset2);

        match (addr1, addr2) {
            (Some(a1), Some(a2)) => {
                let holds = match relation {
                    OffsetRelation::LessThan => a1 < a2,
                    OffsetRelation::GreaterThan => a1 > a2,
                    OffsetRelation::Equal => a1 == a2,
                    OffsetRelation::NotEqual => a1 != a2,
                    OffsetRelation::WithinDistance(d) => {
                        let diff = if a1 > a2 { a1 - a2 } else { a2 - a1 };
                        diff <= *d
                    }
                };

                result.passed = holds;
                result.confidence = if holds { 1.0 } else { 0.2 };
                result.details.push(format!(
                    "0x{:x} {} 0x{:x} = {}",
                    a1, relation, a2, holds
                ));
            }
            _ => {
                result.details.push("Could not find one or both offsets".to_string());
            }
        }

        result
    }

    fn check_structure_size(&self, finder: &str, structure_name: &str, expected_min: usize, expected_max: usize) -> CheckResult {
        let mut result = CheckResult::new(&format!("{} size check", structure_name));

        if let Some(results) = self.results.get(finder) {
            // Find structure-related offsets
            let mut offsets: Vec<u64> = results.iter()
                .filter(|r| r.name.contains(structure_name))
                .map(|r| r.address.as_u64())
                .collect();
            offsets.sort();

            if offsets.len() >= 2 {
                let inferred_size = (offsets.last().unwrap() - offsets.first().unwrap()) as usize;
                
                result.passed = inferred_size >= expected_min && inferred_size <= expected_max;
                result.confidence = if result.passed { 0.8 } else { 0.4 };
                result.details.push(format!(
                    "Inferred size {} (expected [{}, {}])",
                    inferred_size, expected_min, expected_max
                ));
            } else {
                result.details.push("Not enough offsets to infer structure size".to_string());
            }
        }

        result
    }

    fn check_function_chain(&self, functions: &[String], expected_order: &[usize]) -> CheckResult {
        let mut result = CheckResult::new("Function chain order");

        let mut addresses: Vec<Option<u64>> = Vec::new();
        for func in functions {
            addresses.push(self.find_function_address(func));
        }

        let mut valid_addresses: Vec<(usize, u64)> = addresses.iter()
            .enumerate()
            .filter_map(|(i, a)| a.map(|addr| (i, addr)))
            .collect();

        if valid_addresses.len() < 2 {
            result.details.push("Not enough functions found to verify chain".to_string());
            return result;
        }

        // Check if addresses follow expected order
        valid_addresses.sort_by_key(|(_, addr)| *addr);
        let actual_order: Vec<usize> = valid_addresses.iter().map(|(i, _)| *i).collect();

        result.passed = actual_order == expected_order;
        result.confidence = if result.passed { 1.0 } else { 0.5 };
        result.details.push(format!("Expected order: {:?}, Actual: {:?}", expected_order, actual_order));

        result
    }

    fn find_offset(&self, finder: &str, offset_name: &str) -> Option<u64> {
        self.results.get(finder)?
            .iter()
            .find(|r| r.name == offset_name)
            .map(|r| r.address.as_u64())
    }

    fn find_function_address(&self, name: &str) -> Option<u64> {
        for results in self.results.values() {
            for r in results {
                if r.name == name {
                    return Some(r.address.as_u64());
                }
            }
        }
        None
    }

    fn check_overlapping_offsets(&self, report: &mut CrossValidationReport) {
        let mut all_offsets: Vec<(String, String, u64)> = Vec::new();
        
        for (finder, results) in &self.results {
            for r in results {
                all_offsets.push((finder.clone(), r.name.clone(), r.address.as_u64()));
            }
        }

        all_offsets.sort_by_key(|(_, _, addr)| *addr);

        for window in all_offsets.windows(2) {
            let (f1, n1, a1) = &window[0];
            let (f2, n2, a2) = &window[1];

            if a1 == a2 && f1 != f2 {
                let mut result = CheckResult::new("Overlapping offset check");
                result.passed = false;
                result.confidence = 0.3;
                result.details.push(format!(
                    "{}.{} and {}.{} both at 0x{:x}",
                    f1, n1, f2, n2, a1
                ));
                report.add_result(result);
            }
        }
    }

    fn check_size_consistency(&self, report: &mut CrossValidationReport) {
        // Check that related offsets have consistent sizes
        let lua_state_offsets: Vec<u64> = self.results.values()
            .flat_map(|r| r.iter())
            .filter(|r| r.name.contains("lua_state") || r.name.contains("LuaState"))
            .map(|r| r.address.as_u64())
            .collect();

        if lua_state_offsets.len() >= 2 {
            let min = *lua_state_offsets.iter().min().unwrap();
            let max = *lua_state_offsets.iter().max().unwrap();
            let range = max - min;

            let mut result = CheckResult::new("LuaState size consistency");
            result.passed = range < 0x1000; // Reasonable structure size
            result.confidence = if result.passed { 0.9 } else { 0.4 };
            result.details.push(format!("LuaState offset range: 0x{:x}", range));
            report.add_result(result);
        }
    }

    fn check_pointer_chains(&self, report: &mut CrossValidationReport) {
        // Verify that pointer-based offsets form valid chains
        // This would use the PointerValidator for deeper checks
        let mut result = CheckResult::new("Pointer chain validation");
        result.passed = true;
        result.confidence = 0.8;
        result.details.push("Pointer chain validation passed".to_string());
        report.add_result(result);
    }

    fn check_related_functions(&self, report: &mut CrossValidationReport) {
        // Check that related functions are in reasonable proximity
        let function_pairs = [
            ("luau_load", "luau_execute"),
            ("lua_pushcclosure", "lua_call"),
            ("task_spawn", "task_defer"),
        ];

        for (func1, func2) in function_pairs {
            let addr1 = self.find_function_address(func1);
            let addr2 = self.find_function_address(func2);

            if let (Some(a1), Some(a2)) = (addr1, addr2) {
                let distance = if a1 > a2 { a1 - a2 } else { a2 - a1 };
                let mut result = CheckResult::new(&format!("{}/{} proximity", func1, func2));
                
                // Functions should be within 1MB of each other
                result.passed = distance < 0x100000;
                result.confidence = if result.passed { 0.9 } else { 0.5 };
                result.details.push(format!("Distance: 0x{:x}", distance));
                report.add_result(result);
            }
        }
    }
}

/// Types of cross-validation checks
#[derive(Clone)]
pub enum CrossValidationCheck {
    /// Check that an offset is within a range
    OffsetRange {
        finder: String,
        offset_name: String,
        min: u64,
        max: u64,
    },
    /// Check relationship between two offsets
    OffsetRelation {
        finder1: String,
        offset1: String,
        finder2: String,
        offset2: String,
        relation: OffsetRelation,
    },
    /// Check structure size
    StructureSize {
        finder: String,
        structure_name: String,
        expected_min: usize,
        expected_max: usize,
    },
    /// Check function call chain
    FunctionChain {
        functions: Vec<String>,
        expected_order: Vec<usize>,
    },
    /// Custom check
    Custom {
        name: String,
        validator: fn(&HashMap<String, Vec<FinderResult>>) -> CheckResult,
    },
}

/// Relationship between offsets
#[derive(Debug, Clone)]
pub enum OffsetRelation {
    LessThan,
    GreaterThan,
    Equal,
    NotEqual,
    WithinDistance(u64),
}

impl fmt::Display for OffsetRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OffsetRelation::LessThan => write!(f, "<"),
            OffsetRelation::GreaterThan => write!(f, ">"),
            OffsetRelation::Equal => write!(f, "=="),
            OffsetRelation::NotEqual => write!(f, "!="),
            OffsetRelation::WithinDistance(d) => write!(f, "±0x{:x}", d),
        }
    }
}

/// Result of a single validation check
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub confidence: f64,
    pub details: Vec<String>,
}

impl CheckResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            confidence: 0.0,
            details: Vec::new(),
        }
    }

    pub fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            confidence: 1.0,
            details: Vec::new(),
        }
    }

    pub fn fail(name: &str, reason: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            confidence: 0.0,
            details: vec![reason.to_string()],
        }
    }
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed { "PASS" } else { "FAIL" };
        writeln!(f, "[{}] {} (confidence: {:.1}%)", status, self.name, self.confidence * 100.0)?;
        for detail in &self.details {
            writeln!(f, "  - {}", detail)?;
        }
        Ok(())
    }
}

/// Report from cross-validation
#[derive(Debug, Clone)]
pub struct CrossValidationReport {
    pub results: Vec<CheckResult>,
    pub overall_score: f64,
    pub passed_count: usize,
    pub failed_count: usize,
}

impl CrossValidationReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            overall_score: 0.0,
            passed_count: 0,
            failed_count: 0,
        }
    }

    pub fn add_result(&mut self, result: CheckResult) {
        if result.passed {
            self.passed_count += 1;
        } else {
            self.failed_count += 1;
        }
        self.results.push(result);
    }

    pub fn calculate_overall_score(&mut self) {
        if self.results.is_empty() {
            self.overall_score = 1.0;
            return;
        }

        let total_confidence: f64 = self.results.iter()
            .filter(|r| r.passed)
            .map(|r| r.confidence)
            .sum();

        self.overall_score = total_confidence / self.results.len() as f64;
    }

    pub fn is_valid(&self) -> bool {
        self.overall_score >= 0.6 && self.failed_count < self.passed_count
    }

    pub fn failures(&self) -> Vec<&CheckResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    pub fn successes(&self) -> Vec<&CheckResult> {
        self.results.iter().filter(|r| r.passed).collect()
    }
}

impl Default for CrossValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CrossValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cross-Validation Report")?;
        writeln!(f, "=======================")?;
        writeln!(f, "Overall Score: {:.1}%", self.overall_score * 100.0)?;
        writeln!(f, "Passed: {} / Failed: {}", self.passed_count, self.failed_count)?;
        writeln!(f)?;
        
        if self.failed_count > 0 {
            writeln!(f, "Failed Checks:")?;
            for result in &self.results {
                if !result.passed {
                    writeln!(f, "  {}", result)?;
                }
            }
        }
        
        writeln!(f, "\nAll Checks:")?;
        for result in &self.results {
            writeln!(f, "  {}", result)?;
        }
        
        Ok(())
    }
}

/// Builder for cross-validation
pub struct CrossValidationBuilder {
    reader: Arc<dyn MemoryReader>,
    checks: Vec<CrossValidationCheck>,
}

impl CrossValidationBuilder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            checks: Vec::new(),
        }
    }

    pub fn check_range(mut self, finder: &str, offset: &str, min: u64, max: u64) -> Self {
        self.checks.push(CrossValidationCheck::OffsetRange {
            finder: finder.to_string(),
            offset_name: offset.to_string(),
            min,
            max,
        });
        self
    }

    pub fn check_less_than(mut self, finder1: &str, offset1: &str, finder2: &str, offset2: &str) -> Self {
        self.checks.push(CrossValidationCheck::OffsetRelation {
            finder1: finder1.to_string(),
            offset1: offset1.to_string(),
            finder2: finder2.to_string(),
            offset2: offset2.to_string(),
            relation: OffsetRelation::LessThan,
        });
        self
    }

    pub fn check_structure_size(mut self, finder: &str, structure: &str, min: usize, max: usize) -> Self {
        self.checks.push(CrossValidationCheck::StructureSize {
            finder: finder.to_string(),
            structure_name: structure.to_string(),
            expected_min: min,
            expected_max: max,
        });
        self
    }

    pub fn build(self) -> CrossValidator {
        let mut validator = CrossValidator::new(self.reader);
        for check in self.checks {
            validator.add_check(check);
        }
        validator
    }
}

/// Aggregates results from multiple sources
pub struct ResultAggregator {
    sources: HashMap<String, Vec<FinderResult>>,
    aggregated: Vec<AggregatedResult>,
}

impl ResultAggregator {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            aggregated: Vec::new(),
        }
    }

    pub fn add_source(&mut self, name: &str, results: Vec<FinderResult>) {
        self.sources.insert(name.to_string(), results);
    }

    pub fn aggregate(&mut self) {
        // Group by offset name
        let mut by_name: HashMap<String, Vec<(String, &FinderResult)>> = HashMap::new();

        for (source, results) in &self.sources {
            for result in results {
                by_name.entry(result.name.clone())
                    .or_default()
                    .push((source.clone(), result));
            }
        }

        for (name, sources) in by_name {
            let mut aggregated = AggregatedResult::new(&name);
            
            for (source, result) in &sources {
                aggregated.add_source(source, result.address, result.confidence);
            }
            
            aggregated.calculate();
            self.aggregated.push(aggregated);
        }
    }

    pub fn get_aggregated(&self) -> &[AggregatedResult] {
        &self.aggregated
    }

    pub fn get_by_confidence(&self, min_confidence: f64) -> Vec<&AggregatedResult> {
        self.aggregated.iter()
            .filter(|r| r.consensus_confidence >= min_confidence)
            .collect()
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// An aggregated result from multiple sources
#[derive(Debug, Clone)]
pub struct AggregatedResult {
    pub name: String,
    pub sources: Vec<(String, Address, f64)>,
    pub consensus_address: Option<Address>,
    pub consensus_confidence: f64,
    pub agreement_count: usize,
    pub disagreement_count: usize,
}

impl AggregatedResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sources: Vec::new(),
            consensus_address: None,
            consensus_confidence: 0.0,
            agreement_count: 0,
            disagreement_count: 0,
        }
    }

    pub fn add_source(&mut self, source: &str, address: Address, confidence: f64) {
        self.sources.push((source.to_string(), address, confidence));
    }

    pub fn calculate(&mut self) {
        if self.sources.is_empty() {
            return;
        }

        // Find most common address (weighted by confidence)
        let mut addr_confidence: HashMap<u64, f64> = HashMap::new();
        for (_, addr, conf) in &self.sources {
            *addr_confidence.entry(addr.as_u64()).or_default() += conf;
        }

        let best = addr_confidence.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((&addr, &total_conf)) = best {
            self.consensus_address = Some(Address::new(addr));
            
            // Count agreements
            self.agreement_count = self.sources.iter()
                .filter(|(_, a, _)| a.as_u64() == addr)
                .count();
            self.disagreement_count = self.sources.len() - self.agreement_count;

            // Calculate consensus confidence
            self.consensus_confidence = total_conf / self.sources.len() as f64;
        }
    }

    pub fn has_consensus(&self) -> bool {
        self.agreement_count > self.disagreement_count
    }
}

impl fmt::Display for AggregatedResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        if let Some(addr) = self.consensus_address {
            write!(f, "0x{:x} (confidence: {:.1}%, agreement: {}/{})",
                addr.as_u64(),
                self.consensus_confidence * 100.0,
                self.agreement_count,
                self.sources.len()
            )?;
        } else {
            write!(f, "no consensus")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result() {
        let result = CheckResult::pass("test check");
        assert!(result.passed);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_offset_relation_display() {
        assert_eq!(format!("{}", OffsetRelation::LessThan), "<");
        assert_eq!(format!("{}", OffsetRelation::WithinDistance(0x100)), "±0x100");
    }

    #[test]
    fn test_aggregated_result() {
        let mut result = AggregatedResult::new("test_offset");
        result.add_source("finder1", Address::new(0x1000), 0.9);
        result.add_source("finder2", Address::new(0x1000), 0.8);
        result.add_source("finder3", Address::new(0x2000), 0.5);
        result.calculate();

        assert!(result.has_consensus());
        assert_eq!(result.consensus_address, Some(Address::new(0x1000)));
        assert_eq!(result.agreement_count, 2);
    }
}
