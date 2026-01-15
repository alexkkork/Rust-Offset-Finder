// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, OutputStatistics};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

pub struct ReportGenerator {
    format: ReportFormat,
    include_header: bool,
    include_summary: bool,
    include_details: bool,
    group_by_category: bool,
    show_addresses_hex: bool,
    max_items_per_section: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Html,
    Markdown,
    Csv,
}

impl ReportGenerator {
    pub fn new(format: ReportFormat) -> Self {
        Self {
            format,
            include_header: true,
            include_summary: true,
            include_details: true,
            group_by_category: true,
            show_addresses_hex: true,
            max_items_per_section: None,
        }
    }

    pub fn with_header(mut self, include: bool) -> Self {
        self.include_header = include;
        self
    }

    pub fn with_summary(mut self, include: bool) -> Self {
        self.include_summary = include;
        self
    }

    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }

    pub fn with_grouping(mut self, group: bool) -> Self {
        self.group_by_category = group;
        self
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items_per_section = Some(max);
        self
    }

    pub fn generate(&self, output: &OffsetOutput) -> String {
        match self.format {
            ReportFormat::Text => self.generate_text(output),
            ReportFormat::Html => self.generate_html(output),
            ReportFormat::Markdown => self.generate_markdown(output),
            ReportFormat::Csv => self.generate_csv(output),
        }
    }

    pub fn generate_to_file<P: AsRef<Path>>(&self, output: &OffsetOutput, path: P) -> std::io::Result<()> {
        let report = self.generate(output);
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(report.as_bytes())?;
        Ok(())
    }

    fn generate_text(&self, output: &OffsetOutput) -> String {
        let mut report = String::new();

        if self.include_header {
            report.push_str(&self.text_header(output));
            report.push_str("\n\n");
        }

        if self.include_summary {
            report.push_str(&self.text_summary(&output.statistics));
            report.push_str("\n\n");
        }

        if self.include_details {
            report.push_str(&self.text_functions(&output.functions));
            report.push_str("\n\n");
            report.push_str(&self.text_structures(&output.structure_offsets));
            report.push_str("\n\n");
            report.push_str(&self.text_classes(&output.classes));
        }

        report
    }

    fn text_header(&self, output: &OffsetOutput) -> String {
        let mut header = String::new();
        header.push_str("================================================================================\n");
        header.push_str("                    ROBLOX OFFSET GENERATOR REPORT\n");
        header.push_str("================================================================================\n");
        header.push_str(&format!("Target: {}\n", output.target.name));
        header.push_str(&format!("Architecture: {}\n", output.target.architecture));
        header.push_str(&format!("Platform: {}\n", output.target.platform));
        if let Some(version) = &output.target.version {
            header.push_str(&format!("Version: {}\n", version));
        }
        header.push_str(&format!("Base Address: 0x{:x}\n", output.target.base_address));
        header.push_str(&format!("Generated: {}\n", output.generated_at));
        header.push_str("================================================================================");
        header
    }

    fn text_summary(&self, stats: &OutputStatistics) -> String {
        let mut summary = String::new();
        summary.push_str("SUMMARY\n");
        summary.push_str("-------\n");
        summary.push_str(&format!("Total Functions:     {:>8}\n", stats.total_functions));
        summary.push_str(&format!("Total Structures:    {:>8}\n", stats.total_structures));
        summary.push_str(&format!("Total Classes:       {:>8}\n", stats.total_classes));
        summary.push_str(&format!("Total Properties:    {:>8}\n", stats.total_properties));
        summary.push_str(&format!("Total Methods:       {:>8}\n", stats.total_methods));
        summary.push_str(&format!("Total Constants:     {:>8}\n", stats.total_constants));
        summary.push_str(&format!("Avg Confidence:      {:>8.2}%\n", stats.average_confidence * 100.0));
        summary.push_str(&format!("Scan Duration:       {:>8}ms\n", stats.scan_duration_ms));
        summary.push_str(&format!("Memory Scanned:      {:>8} bytes\n", stats.memory_scanned_bytes));
        summary
    }

    fn text_functions(&self, functions: &HashMap<String, FunctionOffset>) -> String {
        let mut text = String::new();
        text.push_str("FUNCTIONS\n");
        text.push_str("---------\n");

        let mut sorted: Vec<_> = functions.iter().collect();
        sorted.sort_by_key(|(name, _)| name.as_str());

        if let Some(max) = self.max_items_per_section {
            sorted.truncate(max);
        }

        if self.group_by_category {
            let mut by_category: HashMap<&str, Vec<_>> = HashMap::new();
            for (name, func) in &sorted {
                by_category.entry(func.category.as_str())
                    .or_insert_with(Vec::new)
                    .push((*name, *func));
            }

            let mut categories: Vec<_> = by_category.keys().collect();
            categories.sort();

            for category in categories {
                text.push_str(&format!("\n[{}]\n", category));
                for (name, func) in &by_category[category] {
                    let addr_str = if self.show_addresses_hex {
                        format!("0x{:016x}", func.address)
                    } else {
                        format!("{}", func.address)
                    };
                    text.push_str(&format!("  {} = {} ({:.1}%)\n", name, addr_str, func.confidence * 100.0));
                }
            }
        } else {
            for (name, func) in sorted {
                let addr_str = if self.show_addresses_hex {
                    format!("0x{:016x}", func.address)
                } else {
                    format!("{}", func.address)
                };
                text.push_str(&format!("{} = {} ({:.1}%)\n", name, addr_str, func.confidence * 100.0));
            }
        }

        text
    }

    fn text_structures(&self, structures: &HashMap<String, StructureOffsets>) -> String {
        let mut text = String::new();
        text.push_str("STRUCTURES\n");
        text.push_str("----------\n");

        let mut sorted: Vec<_> = structures.iter().collect();
        sorted.sort_by_key(|(name, _)| name.as_str());

        if let Some(max) = self.max_items_per_section {
            sorted.truncate(max);
        }

        for (name, structure) in sorted {
            text.push_str(&format!("\n{} (size: {}, align: {})\n", name, structure.size, structure.alignment));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                text.push_str(&format!("  +0x{:04x} {} ({})\n", field.offset, field_name, field.field_type));
            }
        }

        text
    }

    fn text_classes(&self, classes: &[ClassOffset]) -> String {
        let mut text = String::new();
        text.push_str("CLASSES\n");
        text.push_str("-------\n");

        let mut sorted = classes.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(max) = self.max_items_per_section {
            sorted.truncate(max);
        }

        for class in sorted {
            text.push_str(&format!("\n{}", class.name));
            if let Some(parent) = &class.parent {
                text.push_str(&format!(" : {}", parent));
            }
            text.push('\n');

            if let Some(vtable) = class.vtable_address {
                text.push_str(&format!("  VTable: 0x{:016x}\n", vtable));
            }
            text.push_str(&format!("  Size: {} bytes\n", class.size));

            if !class.properties.is_empty() {
                text.push_str(&format!("  Properties: {}\n", class.properties.join(", ")));
            }
            if !class.methods.is_empty() {
                text.push_str(&format!("  Methods: {}\n", class.methods.join(", ")));
            }
        }

        text
    }

    fn generate_html(&self, output: &OffsetOutput) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("<title>Roblox Offset Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: 'Courier New', monospace; background: #1a1a2e; color: #eee; padding: 20px; }\n");
        html.push_str("h1, h2, h3 { color: #00d4ff; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #444; padding: 8px; text-align: left; }\n");
        html.push_str("th { background: #2a2a4e; color: #00d4ff; }\n");
        html.push_str("tr:nth-child(even) { background: #2a2a3e; }\n");
        html.push_str("tr:hover { background: #3a3a5e; }\n");
        html.push_str(".address { color: #ff6b6b; }\n");
        html.push_str(".confidence { color: #4ecdc4; }\n");
        html.push_str(".category { color: #ffe66d; }\n");
        html.push_str(".summary-box { background: #2a2a4e; padding: 15px; border-radius: 5px; margin: 20px 0; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        if self.include_header {
            html.push_str(&format!("<h1>Roblox Offset Report</h1>\n"));
            html.push_str(&format!("<p>Target: {} | Architecture: {} | Platform: {}</p>\n",
                output.target.name, output.target.architecture, output.target.platform));
            html.push_str(&format!("<p>Base Address: <span class=\"address\">0x{:x}</span></p>\n", output.target.base_address));
        }

        if self.include_summary {
            html.push_str("<div class=\"summary-box\">\n");
            html.push_str("<h2>Summary</h2>\n");
            html.push_str(&format!("<p>Functions: {} | Structures: {} | Classes: {} | Methods: {}</p>\n",
                output.statistics.total_functions,
                output.statistics.total_structures,
                output.statistics.total_classes,
                output.statistics.total_methods));
            html.push_str(&format!("<p>Average Confidence: <span class=\"confidence\">{:.1}%</span></p>\n",
                output.statistics.average_confidence * 100.0));
            html.push_str("</div>\n");
        }

        if self.include_details {
            html.push_str("<h2>Functions</h2>\n");
            html.push_str("<table>\n<tr><th>Name</th><th>Address</th><th>Confidence</th><th>Category</th></tr>\n");

            let mut sorted: Vec<_> = output.functions.iter().collect();
            sorted.sort_by_key(|(name, _)| name.as_str());

            for (name, func) in sorted {
                html.push_str(&format!(
                    "<tr><td>{}</td><td class=\"address\">0x{:x}</td><td class=\"confidence\">{:.1}%</td><td class=\"category\">{}</td></tr>\n",
                    name, func.address, func.confidence * 100.0, func.category
                ));
            }
            html.push_str("</table>\n");

            html.push_str("<h2>Structures</h2>\n");
            for (name, structure) in &output.structure_offsets {
                html.push_str(&format!("<h3>{}</h3>\n", name));
                html.push_str("<table>\n<tr><th>Field</th><th>Offset</th><th>Size</th><th>Type</th></tr>\n");

                let mut fields: Vec<_> = structure.fields.iter().collect();
                fields.sort_by_key(|(_, f)| f.offset);

                for (field_name, field) in fields {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td class=\"address\">0x{:x}</td><td>{}</td><td>{}</td></tr>\n",
                        field_name, field.offset, field.size, field.field_type
                    ));
                }
                html.push_str("</table>\n");
            }
        }

        html.push_str("</body>\n</html>");
        html
    }

    fn generate_markdown(&self, output: &OffsetOutput) -> String {
        let mut md = String::new();

        if self.include_header {
            md.push_str("# Roblox Offset Report\n\n");
            md.push_str(&format!("- **Target:** {}\n", output.target.name));
            md.push_str(&format!("- **Architecture:** {}\n", output.target.architecture));
            md.push_str(&format!("- **Platform:** {}\n", output.target.platform));
            md.push_str(&format!("- **Base Address:** `0x{:x}`\n\n", output.target.base_address));
        }

        if self.include_summary {
            md.push_str("## Summary\n\n");
            md.push_str(&format!("| Metric | Value |\n"));
            md.push_str(&format!("|--------|-------|\n"));
            md.push_str(&format!("| Functions | {} |\n", output.statistics.total_functions));
            md.push_str(&format!("| Structures | {} |\n", output.statistics.total_structures));
            md.push_str(&format!("| Classes | {} |\n", output.statistics.total_classes));
            md.push_str(&format!("| Methods | {} |\n", output.statistics.total_methods));
            md.push_str(&format!("| Avg Confidence | {:.1}% |\n\n", output.statistics.average_confidence * 100.0));
        }

        if self.include_details {
            md.push_str("## Functions\n\n");
            md.push_str("| Name | Address | Confidence | Category |\n");
            md.push_str("|------|---------|------------|----------|\n");

            let mut sorted: Vec<_> = output.functions.iter().collect();
            sorted.sort_by_key(|(name, _)| name.as_str());

            for (name, func) in sorted {
                md.push_str(&format!("| {} | `0x{:x}` | {:.1}% | {} |\n",
                    name, func.address, func.confidence * 100.0, func.category));
            }

            md.push_str("\n## Structures\n\n");
            for (name, structure) in &output.structure_offsets {
                md.push_str(&format!("### {}\n\n", name));
                md.push_str("| Field | Offset | Size | Type |\n");
                md.push_str("|-------|--------|------|------|\n");

                let mut fields: Vec<_> = structure.fields.iter().collect();
                fields.sort_by_key(|(_, f)| f.offset);

                for (field_name, field) in fields {
                    md.push_str(&format!("| {} | `0x{:x}` | {} | {} |\n",
                        field_name, field.offset, field.size, field.field_type));
                }
                md.push_str("\n");
            }
        }

        md
    }

    fn generate_csv(&self, output: &OffsetOutput) -> String {
        let mut csv = String::new();

        csv.push_str("Type,Name,Address,Confidence,Category,Extra\n");

        for (name, func) in &output.functions {
            csv.push_str(&format!("function,{},0x{:x},{:.4},{},{}\n",
                name, func.address, func.confidence, func.category, func.discovery_method));
        }

        for (struct_name, structure) in &output.structure_offsets {
            for (field_name, field) in &structure.fields {
                csv.push_str(&format!("field,{}.{},0x{:x},,{},{}\n",
                    struct_name, field_name, field.offset, field.field_type, field.size));
            }
        }

        for class in &output.classes {
            let vtable_str = class.vtable_address.map(|a| format!("0x{:x}", a)).unwrap_or_default();
            csv.push_str(&format!("class,{},{},,,{}\n",
                class.name, vtable_str, class.size));
        }

        csv
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new(ReportFormat::Text)
    }
}

pub fn generate_text_report(output: &OffsetOutput) -> String {
    ReportGenerator::new(ReportFormat::Text).generate(output)
}

pub fn generate_html_report(output: &OffsetOutput) -> String {
    ReportGenerator::new(ReportFormat::Html).generate(output)
}

pub fn generate_markdown_report(output: &OffsetOutput) -> String {
    ReportGenerator::new(ReportFormat::Markdown).generate(output)
}

pub fn generate_csv_report(output: &OffsetOutput) -> String {
    ReportGenerator::new(ReportFormat::Csv).generate(output)
}
