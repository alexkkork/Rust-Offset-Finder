// Tue Jan 13 2026 - Alex

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub overall_score: f64,
    pub category_scores: HashMap<String, f64>,
    pub summary: ValidationSummary,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            overall_score: 100.0,
            category_scores: HashMap::new(),
            summary: ValidationSummary::new(),
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.summary.add_issue(&issue);
        self.issues.push(issue);
    }

    pub fn add_issues(&mut self, issues: Vec<ValidationIssue>) {
        for issue in issues {
            self.add_issue(issue);
        }
    }

    pub fn calculate_overall_score(&mut self) {
        let mut score = 100.0;

        for issue in &self.issues {
            match issue.severity {
                IssueSeverity::Error => score -= 10.0,
                IssueSeverity::Warning => score -= 5.0,
                IssueSeverity::Info => score -= 1.0,
            }
        }

        self.overall_score = score.max(0.0);
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning)
    }

    pub fn infos(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Info)
    }

    pub fn is_valid(&self) -> bool {
        self.summary.error_count == 0
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Validation Report ===\n"));
        output.push_str(&format!("Overall Score: {:.1}%\n", self.overall_score));
        output.push_str(&format!("\nSummary:\n"));
        output.push_str(&format!("  Errors: {}\n", self.summary.error_count));
        output.push_str(&format!("  Warnings: {}\n", self.summary.warning_count));
        output.push_str(&format!("  Info: {}\n", self.summary.info_count));

        if !self.issues.is_empty() {
            output.push_str(&format!("\nIssues:\n"));

            let errors: Vec<_> = self.errors().collect();
            if !errors.is_empty() {
                output.push_str("  [ERRORS]\n");
                for issue in errors {
                    output.push_str(&format!("    - [{}] {}: {}\n", 
                        issue.category, issue.item_name, issue.message));
                    if let Some(suggestion) = &issue.suggestion {
                        output.push_str(&format!("      Suggestion: {}\n", suggestion));
                    }
                }
            }

            let warnings: Vec<_> = self.warnings().collect();
            if !warnings.is_empty() {
                output.push_str("  [WARNINGS]\n");
                for issue in warnings {
                    output.push_str(&format!("    - [{}] {}: {}\n",
                        issue.category, issue.item_name, issue.message));
                }
            }

            let infos: Vec<_> = self.infos().collect();
            if !infos.is_empty() {
                output.push_str("  [INFO]\n");
                for issue in infos {
                    output.push_str(&format!("    - [{}] {}: {}\n",
                        issue.category, issue.item_name, issue.message));
                }
            }
        }

        output
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serialize for ValidationReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ValidationReport", 4)?;
        state.serialize_field("overall_score", &self.overall_score)?;
        state.serialize_field("error_count", &self.summary.error_count)?;
        state.serialize_field("warning_count", &self.summary.warning_count)?;
        state.serialize_field("info_count", &self.summary.info_count)?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub category: String,
    pub item_name: String,
    pub message: String,
    pub severity: IssueSeverity,
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    pub fn error(category: &str, item: &str, message: &str) -> Self {
        Self {
            category: category.to_string(),
            item_name: item.to_string(),
            message: message.to_string(),
            severity: IssueSeverity::Error,
            suggestion: None,
        }
    }

    pub fn warning(category: &str, item: &str, message: &str) -> Self {
        Self {
            category: category.to_string(),
            item_name: item.to_string(),
            message: message.to_string(),
            severity: IssueSeverity::Warning,
            suggestion: None,
        }
    }

    pub fn info(category: &str, item: &str, message: &str) -> Self {
        Self {
            category: category.to_string(),
            item_name: item.to_string(),
            message: message.to_string(),
            severity: IssueSeverity::Info,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl IssueSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "ERROR",
            IssueSeverity::Warning => "WARNING",
            IssueSeverity::Info => "INFO",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "\x1b[31m",
            IssueSeverity::Warning => "\x1b[33m",
            IssueSeverity::Info => "\x1b[34m",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub categories_affected: Vec<String>,
}

impl ValidationSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_issue(&mut self, issue: &ValidationIssue) {
        match issue.severity {
            IssueSeverity::Error => self.error_count += 1,
            IssueSeverity::Warning => self.warning_count += 1,
            IssueSeverity::Info => self.info_count += 1,
        }

        if !self.categories_affected.contains(&issue.category) {
            self.categories_affected.push(issue.category.clone());
        }
    }

    pub fn total_issues(&self) -> usize {
        self.error_count + self.warning_count + self.info_count
    }

    pub fn is_clean(&self) -> bool {
        self.total_issues() == 0
    }
}
