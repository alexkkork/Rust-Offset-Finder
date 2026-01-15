// Tue Jan 15 2026 - Alex

use crate::diff::binary::BinaryDiff;
use crate::diff::offset::{OffsetDiff, OffsetChangeKind};
use crate::diff::version::VersionComparison;
use std::fmt;
use std::io::Write;

/// Complete diff report
#[derive(Debug, Clone)]
pub struct DiffReport {
    pub title: String,
    pub old_version: String,
    pub new_version: String,
    pub timestamp: String,
    pub binary_diff: Option<BinaryDiff>,
    pub offset_diff: Option<OffsetDiff>,
    pub version_comparison: Option<VersionComparison>,
    pub summary: DiffSummary,
    pub sections: Vec<ReportSection>,
}

impl DiffReport {
    pub fn new(old_ver: &str, new_ver: &str) -> Self {
        Self {
            title: format!("Diff Report: {} -> {}", old_ver, new_ver),
            old_version: old_ver.to_string(),
            new_version: new_ver.to_string(),
            timestamp: chrono_lite::now(),
            binary_diff: None,
            offset_diff: None,
            version_comparison: None,
            summary: DiffSummary::default(),
            sections: Vec::new(),
        }
    }

    pub fn with_binary_diff(mut self, diff: BinaryDiff) -> Self {
        self.summary.binary_changes = diff.change_count();
        self.summary.changed_regions = diff.region_count();
        self.binary_diff = Some(diff);
        self
    }

    pub fn with_offset_diff(mut self, diff: OffsetDiff) -> Self {
        self.summary.offset_changes = diff.change_count();
        self.summary.offsets_unchanged = diff.unchanged_count();
        self.offset_diff = Some(diff);
        self
    }

    pub fn with_version_comparison(mut self, comp: VersionComparison) -> Self {
        self.version_comparison = Some(comp);
        self
    }

    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }

    /// Export to specified format
    pub fn export(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Text => self.to_text(),
            ReportFormat::Markdown => self.to_markdown(),
            ReportFormat::Html => self.to_html(),
            ReportFormat::Json => self.to_json(),
        }
    }

    fn to_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", self.title));
        output.push_str(&"=".repeat(self.title.len()));
        output.push_str("\n\n");
        
        output.push_str(&format!("Generated: {}\n", self.timestamp));
        output.push_str(&format!("Old Version: {}\n", self.old_version));
        output.push_str(&format!("New Version: {}\n\n", self.new_version));
        
        output.push_str("Summary\n-------\n");
        output.push_str(&format!("{}\n", self.summary));
        
        if let Some(ref diff) = self.offset_diff {
            output.push_str("\nOffset Changes\n--------------\n");
            for change in &diff.changes {
                output.push_str(&format!("  {}\n", change));
            }
        }
        
        for section in &self.sections {
            output.push_str(&format!("\n{}\n", section.title));
            output.push_str(&"-".repeat(section.title.len()));
            output.push_str("\n");
            output.push_str(&section.content);
            output.push_str("\n");
        }
        
        output
    }

    fn to_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("**Generated:** {}\n\n", self.timestamp));
        output.push_str(&format!("| Version | Value |\n"));
        output.push_str(&format!("|---------|-------|\n"));
        output.push_str(&format!("| Old | {} |\n", self.old_version));
        output.push_str(&format!("| New | {} |\n\n", self.new_version));
        
        output.push_str("## Summary\n\n");
        output.push_str(&format!("- **Offset Changes:** {}\n", self.summary.offset_changes));
        output.push_str(&format!("- **Offsets Unchanged:** {}\n", self.summary.offsets_unchanged));
        output.push_str(&format!("- **Binary Changes:** {}\n", self.summary.binary_changes));
        output.push_str(&format!("- **Changed Regions:** {}\n\n", self.summary.changed_regions));
        
        if let Some(ref diff) = self.offset_diff {
            if !diff.changes.is_empty() {
                output.push_str("## Offset Changes\n\n");
                output.push_str("| Name | Old | New | Delta |\n");
                output.push_str("|------|-----|-----|-------|\n");
                
                for change in &diff.changes {
                    let delta_str = match change.kind {
                        OffsetChangeKind::ValueChanged => format!("{:+}", change.delta),
                        OffsetChangeKind::Added => "NEW".to_string(),
                        OffsetChangeKind::Removed => "REMOVED".to_string(),
                        OffsetChangeKind::TypeChanged => "TYPE".to_string(),
                    };
                    output.push_str(&format!("| {} | {} | {} | {} |\n",
                        change.name, change.old_hex(), change.new_hex(), delta_str));
                }
                output.push_str("\n");
            }
        }
        
        for section in &self.sections {
            output.push_str(&format!("## {}\n\n", section.title));
            output.push_str(&section.content);
            output.push_str("\n\n");
        }
        
        output
    }

    fn to_html(&self) -> String {
        let mut output = String::new();
        
        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str("<meta charset=\"utf-8\">\n");
        output.push_str(&format!("<title>{}</title>\n", self.title));
        output.push_str("<style>\n");
        output.push_str("body { font-family: -apple-system, sans-serif; margin: 2em; }\n");
        output.push_str("table { border-collapse: collapse; width: 100%; }\n");
        output.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        output.push_str("th { background: #f5f5f5; }\n");
        output.push_str(".added { color: green; }\n");
        output.push_str(".removed { color: red; }\n");
        output.push_str(".changed { color: orange; }\n");
        output.push_str("</style>\n</head>\n<body>\n");
        
        output.push_str(&format!("<h1>{}</h1>\n", self.title));
        output.push_str(&format!("<p><strong>Generated:</strong> {}</p>\n", self.timestamp));
        
        output.push_str("<h2>Summary</h2>\n");
        output.push_str("<ul>\n");
        output.push_str(&format!("<li>Offset Changes: {}</li>\n", self.summary.offset_changes));
        output.push_str(&format!("<li>Unchanged: {}</li>\n", self.summary.offsets_unchanged));
        output.push_str("</ul>\n");
        
        if let Some(ref diff) = self.offset_diff {
            if !diff.changes.is_empty() {
                output.push_str("<h2>Offset Changes</h2>\n");
                output.push_str("<table>\n<tr><th>Name</th><th>Old</th><th>New</th><th>Delta</th></tr>\n");
                
                for change in &diff.changes {
                    let class = match change.kind {
                        OffsetChangeKind::Added => "added",
                        OffsetChangeKind::Removed => "removed",
                        _ => "changed",
                    };
                    output.push_str(&format!("<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{:+}</td></tr>\n",
                        class, change.name, change.old_hex(), change.new_hex(), change.delta));
                }
                
                output.push_str("</table>\n");
            }
        }
        
        output.push_str("</body>\n</html>");
        
        output
    }

    fn to_json(&self) -> String {
        let mut json = String::new();
        
        json.push_str("{\n");
        json.push_str(&format!("  \"title\": \"{}\",\n", self.title));
        json.push_str(&format!("  \"old_version\": \"{}\",\n", self.old_version));
        json.push_str(&format!("  \"new_version\": \"{}\",\n", self.new_version));
        json.push_str(&format!("  \"timestamp\": \"{}\",\n", self.timestamp));
        
        json.push_str("  \"summary\": {\n");
        json.push_str(&format!("    \"offset_changes\": {},\n", self.summary.offset_changes));
        json.push_str(&format!("    \"offsets_unchanged\": {},\n", self.summary.offsets_unchanged));
        json.push_str(&format!("    \"binary_changes\": {},\n", self.summary.binary_changes));
        json.push_str(&format!("    \"changed_regions\": {}\n", self.summary.changed_regions));
        json.push_str("  },\n");
        
        json.push_str("  \"changes\": [\n");
        if let Some(ref diff) = self.offset_diff {
            for (i, change) in diff.changes.iter().enumerate() {
                json.push_str("    {\n");
                json.push_str(&format!("      \"name\": \"{}\",\n", change.name));
                json.push_str(&format!("      \"old_value\": {},\n", 
                    change.old_value.map(|v| v.to_string()).unwrap_or("null".to_string())));
                json.push_str(&format!("      \"new_value\": {},\n",
                    change.new_value.map(|v| v.to_string()).unwrap_or("null".to_string())));
                json.push_str(&format!("      \"delta\": {},\n", change.delta));
                json.push_str(&format!("      \"kind\": \"{:?}\"\n", change.kind));
                json.push_str("    }");
                if i < diff.changes.len() - 1 {
                    json.push(',');
                }
                json.push('\n');
            }
        }
        json.push_str("  ]\n");
        
        json.push_str("}\n");
        
        json
    }

    /// Write report to file
    pub fn write_to_file(&self, path: &str, format: ReportFormat) -> std::io::Result<()> {
        let content = self.export(format);
        let mut file = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl fmt::Display for DiffReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

/// Report section
#[derive(Debug, Clone)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
}

impl ReportSection {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
        }
    }
}

/// Summary of diff
#[derive(Debug, Clone, Default)]
pub struct DiffSummary {
    pub offset_changes: usize,
    pub offsets_unchanged: usize,
    pub binary_changes: usize,
    pub changed_regions: usize,
    pub functions_changed: usize,
    pub breaking_changes: usize,
}

impl fmt::Display for DiffSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Offset changes: {}", self.offset_changes)?;
        writeln!(f, "Unchanged: {}", self.offsets_unchanged)?;
        writeln!(f, "Binary changes: {}", self.binary_changes)?;
        writeln!(f, "Changed regions: {}", self.changed_regions)?;
        if self.breaking_changes > 0 {
            writeln!(f, "Breaking changes: {}", self.breaking_changes)?;
        }
        Ok(())
    }
}

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Markdown,
    Html,
    Json,
}

impl ReportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Text => "txt",
            ReportFormat::Markdown => "md",
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
        }
    }
}

/// Builder for diff reports
pub struct DiffReportBuilder {
    report: DiffReport,
}

impl DiffReportBuilder {
    pub fn new(old_ver: &str, new_ver: &str) -> Self {
        Self {
            report: DiffReport::new(old_ver, new_ver),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.report.title = title.to_string();
        self
    }

    pub fn binary_diff(mut self, diff: BinaryDiff) -> Self {
        self.report = self.report.with_binary_diff(diff);
        self
    }

    pub fn offset_diff(mut self, diff: OffsetDiff) -> Self {
        self.report = self.report.with_offset_diff(diff);
        self
    }

    pub fn section(mut self, title: &str, content: &str) -> Self {
        self.report.add_section(ReportSection::new(title, content));
        self
    }

    pub fn build(self) -> DiffReport {
        self.report
    }
}

/// Simple timestamp helper (since we don't have chrono)
mod chrono_lite {
    pub fn now() -> String {
        // Return a placeholder - in real code would use actual timestamp
        "2026-01-15".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_builder() {
        let report = DiffReportBuilder::new("v1", "v2")
            .title("Test Report")
            .section("Notes", "Some notes")
            .build();

        assert_eq!(report.old_version, "v1");
        assert_eq!(report.new_version, "v2");
    }

    #[test]
    fn test_export_formats() {
        let report = DiffReport::new("v1", "v2");
        
        let text = report.export(ReportFormat::Text);
        assert!(text.contains("v1"));
        
        let json = report.export(ReportFormat::Json);
        assert!(json.contains("\"old_version\""));
    }
}
