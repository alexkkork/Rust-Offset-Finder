// Tue Jan 13 2026 - Alex

use crate::validation::validator::{ValidationResult, ValidationIssue, IssueSeverity};
use std::collections::HashMap;
use std::fmt;

pub struct ValidationReport {
    function_results: HashMap<String, ValidationResult>,
    structure_results: HashMap<String, HashMap<String, ValidationResult>>,
    class_results: HashMap<String, ValidationResult>,
    constant_results: HashMap<String, ValidationResult>,
    summary: Option<ValidationSummary>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            function_results: HashMap::new(),
            structure_results: HashMap::new(),
            class_results: HashMap::new(),
            constant_results: HashMap::new(),
            summary: None,
        }
    }

    pub fn add_function_result(&mut self, name: String, result: ValidationResult) {
        self.function_results.insert(name, result);
    }

    pub fn add_structure_result(&mut self, struct_name: String, field_name: String, result: ValidationResult) {
        self.structure_results
            .entry(struct_name)
            .or_default()
            .insert(field_name, result);
    }

    pub fn add_class_result(&mut self, name: String, result: ValidationResult) {
        self.class_results.insert(name, result);
    }

    pub fn add_constant_result(&mut self, name: String, result: ValidationResult) {
        self.constant_results.insert(name, result);
    }

    pub fn calculate_summary(&mut self) {
        let mut total = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut warnings = 0;
        let mut total_confidence = 0.0;

        for result in self.function_results.values() {
            total += 1;
            total_confidence += result.confidence;
            if result.valid {
                passed += 1;
            } else {
                failed += 1;
            }
            if result.has_issues() && result.valid {
                warnings += 1;
            }
        }

        for fields in self.structure_results.values() {
            for result in fields.values() {
                total += 1;
                total_confidence += result.confidence;
                if result.valid {
                    passed += 1;
                } else {
                    failed += 1;
                }
                if result.has_issues() && result.valid {
                    warnings += 1;
                }
            }
        }

        for result in self.class_results.values() {
            total += 1;
            total_confidence += result.confidence;
            if result.valid {
                passed += 1;
            } else {
                failed += 1;
            }
            if result.has_issues() && result.valid {
                warnings += 1;
            }
        }

        for result in self.constant_results.values() {
            total += 1;
            total_confidence += result.confidence;
            if result.valid {
                passed += 1;
            } else {
                failed += 1;
            }
            if result.has_issues() && result.valid {
                warnings += 1;
            }
        }

        let average_confidence = if total > 0 {
            total_confidence / total as f64
        } else {
            0.0
        };

        self.summary = Some(ValidationSummary {
            total_validations: total,
            passed,
            failed,
            warnings,
            average_confidence,
            issues_by_severity: self.count_issues_by_severity(),
        });
    }

    fn count_issues_by_severity(&self) -> HashMap<IssueSeverity, usize> {
        let mut counts: HashMap<IssueSeverity, usize> = HashMap::new();

        let all_results: Vec<&ValidationResult> = self.function_results.values()
            .chain(self.class_results.values())
            .chain(self.constant_results.values())
            .chain(self.structure_results.values().flat_map(|m| m.values()))
            .collect();

        for result in all_results {
            for issue in &result.issues {
                *counts.entry(issue.severity()).or_insert(0) += 1;
            }
        }

        counts
    }

    pub fn summary(&self) -> Option<&ValidationSummary> {
        self.summary.as_ref()
    }

    pub fn is_all_valid(&self) -> bool {
        self.summary.as_ref().map(|s| s.failed == 0).unwrap_or(false)
    }

    pub fn pass_rate(&self) -> f64 {
        self.summary.as_ref()
            .map(|s| {
                if s.total_validations > 0 {
                    s.passed as f64 / s.total_validations as f64
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }

    pub fn get_function_result(&self, name: &str) -> Option<&ValidationResult> {
        self.function_results.get(name)
    }

    pub fn get_structure_result(&self, struct_name: &str, field_name: &str) -> Option<&ValidationResult> {
        self.structure_results
            .get(struct_name)
            .and_then(|fields| fields.get(field_name))
    }

    pub fn get_class_result(&self, name: &str) -> Option<&ValidationResult> {
        self.class_results.get(name)
    }

    pub fn get_constant_result(&self, name: &str) -> Option<&ValidationResult> {
        self.constant_results.get(name)
    }

    pub fn failed_functions(&self) -> Vec<(&String, &ValidationResult)> {
        self.function_results.iter()
            .filter(|(_, r)| !r.valid)
            .collect()
    }

    pub fn failed_structures(&self) -> Vec<(&String, &String, &ValidationResult)> {
        self.structure_results.iter()
            .flat_map(|(struct_name, fields)| {
                fields.iter()
                    .filter(|(_, r)| !r.valid)
                    .map(move |(field_name, r)| (struct_name, field_name, r))
            })
            .collect()
    }

    pub fn critical_issues(&self) -> Vec<(String, &ValidationIssue)> {
        let mut critical = Vec::new();

        for (name, result) in &self.function_results {
            for issue in &result.issues {
                if issue.severity() == IssueSeverity::Critical {
                    critical.push((format!("Function: {}", name), issue));
                }
            }
        }

        for (struct_name, fields) in &self.structure_results {
            for (field_name, result) in fields {
                for issue in &result.issues {
                    if issue.severity() == IssueSeverity::Critical {
                        critical.push((format!("Structure: {}.{}", struct_name, field_name), issue));
                    }
                }
            }
        }

        for (name, result) in &self.class_results {
            for issue in &result.issues {
                if issue.severity() == IssueSeverity::Critical {
                    critical.push((format!("Class: {}", name), issue));
                }
            }
        }

        for (name, result) in &self.constant_results {
            for issue in &result.issues {
                if issue.severity() == IssueSeverity::Critical {
                    critical.push((format!("Constant: {}", name), issue));
                }
            }
        }

        critical
    }

    pub fn to_text_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Validation Report ===\n\n");

        if let Some(summary) = &self.summary {
            report.push_str(&format!("Summary:\n"));
            report.push_str(&format!("  Total Validations: {}\n", summary.total_validations));
            report.push_str(&format!("  Passed: {}\n", summary.passed));
            report.push_str(&format!("  Failed: {}\n", summary.failed));
            report.push_str(&format!("  Warnings: {}\n", summary.warnings));
            report.push_str(&format!("  Average Confidence: {:.2}%\n", summary.average_confidence * 100.0));
            report.push_str("\n");
        }

        if !self.function_results.is_empty() {
            report.push_str("Function Validations:\n");
            for (name, result) in &self.function_results {
                let status = if result.valid { "PASS" } else { "FAIL" };
                report.push_str(&format!("  {} [{}] confidence={:.2}%\n",
                    name, status, result.confidence * 100.0));

                for issue in &result.issues {
                    report.push_str(&format!("    - {:?}: {}\n",
                        issue.severity(), issue.description()));
                }
            }
            report.push_str("\n");
        }

        if !self.structure_results.is_empty() {
            report.push_str("Structure Validations:\n");
            for (struct_name, fields) in &self.structure_results {
                report.push_str(&format!("  {}:\n", struct_name));
                for (field_name, result) in fields {
                    let status = if result.valid { "PASS" } else { "FAIL" };
                    report.push_str(&format!("    {} [{}] confidence={:.2}%\n",
                        field_name, status, result.confidence * 100.0));

                    for issue in &result.issues {
                        report.push_str(&format!("      - {:?}: {}\n",
                            issue.severity(), issue.description()));
                    }
                }
            }
            report.push_str("\n");
        }

        report
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text_report())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_validations: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub average_confidence: f64,
    pub issues_by_severity: HashMap<IssueSeverity, usize>,
}

impl ValidationSummary {
    pub fn success_rate(&self) -> f64 {
        if self.total_validations > 0 {
            self.passed as f64 / self.total_validations as f64
        } else {
            0.0
        }
    }

    pub fn critical_issues(&self) -> usize {
        *self.issues_by_severity.get(&IssueSeverity::Critical).unwrap_or(&0)
    }

    pub fn error_issues(&self) -> usize {
        *self.issues_by_severity.get(&IssueSeverity::Error).unwrap_or(&0)
    }

    pub fn warning_issues(&self) -> usize {
        *self.issues_by_severity.get(&IssueSeverity::Warning).unwrap_or(&0)
    }

    pub fn info_issues(&self) -> usize {
        *self.issues_by_severity.get(&IssueSeverity::Info).unwrap_or(&0)
    }
}

pub struct DetailedReport {
    pub report: ValidationReport,
    pub timestamp: String,
    pub version: String,
    pub target_info: TargetInfo,
}

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub binary_path: Option<String>,
    pub process_name: Option<String>,
    pub architecture: String,
    pub os: String,
}

impl DetailedReport {
    pub fn new(report: ValidationReport) -> Self {
        Self {
            report,
            timestamp: chrono_lite_timestamp(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            target_info: TargetInfo {
                binary_path: None,
                process_name: None,
                architecture: "arm64".to_string(),
                os: "macOS".to_string(),
            },
        }
    }

    pub fn with_target(mut self, target: TargetInfo) -> Self {
        self.target_info = target;
        self
    }
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}", duration.as_secs())
}
