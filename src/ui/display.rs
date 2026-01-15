// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset};
use colored::*;
use std::collections::HashMap;

pub struct DisplayRenderer {
    use_color: bool,
    use_unicode: bool,
    compact_mode: bool,
    max_items: Option<usize>,
    address_format: AddressDisplayFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressDisplayFormat {
    Full,
    Short,
    Relative,
}

impl DisplayRenderer {
    pub fn new() -> Self {
        Self {
            use_color: true,
            use_unicode: true,
            compact_mode: false,
            max_items: None,
            address_format: AddressDisplayFormat::Full,
        }
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn with_unicode(mut self, use_unicode: bool) -> Self {
        self.use_unicode = use_unicode;
        self
    }

    pub fn with_compact(mut self, compact: bool) -> Self {
        self.compact_mode = compact;
        self
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn with_address_format(mut self, format: AddressDisplayFormat) -> Self {
        self.address_format = format;
        self
    }

    pub fn format_address(&self, address: u64, base: u64) -> String {
        match self.address_format {
            AddressDisplayFormat::Full => format!("0x{:016x}", address),
            AddressDisplayFormat::Short => format!("0x{:x}", address),
            AddressDisplayFormat::Relative => {
                if address >= base {
                    format!("+0x{:x}", address - base)
                } else {
                    format!("-0x{:x}", base - address)
                }
            }
        }
    }

    pub fn render_function(&self, name: &str, func: &FunctionOffset, base: u64) -> String {
        let addr_str = self.format_address(func.address, base);
        let conf_str = format!("{:.1}%", func.confidence * 100.0);

        if self.use_color {
            format!("{} {} {} [{}]",
                name.cyan(),
                addr_str.red(),
                conf_str.green(),
                func.category.yellow()
            )
        } else {
            format!("{} {} {} [{}]", name, addr_str, conf_str, func.category)
        }
    }

    pub fn render_function_list(&self, functions: &HashMap<String, FunctionOffset>, base: u64) -> String {
        let mut lines = Vec::new();
        let mut sorted: Vec<_> = functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        if let Some(max) = self.max_items {
            sorted.truncate(max);
        }

        let header = if self.compact_mode {
            format!("Functions ({})", functions.len())
        } else {
            format!("Found {} functions:", functions.len())
        };

        if self.use_color {
            lines.push(header.cyan().bold().to_string());
        } else {
            lines.push(header);
        }

        for (name, func) in sorted {
            lines.push(format!("  {}", self.render_function(name, func, base)));
        }

        if let Some(max) = self.max_items {
            if functions.len() > max {
                lines.push(format!("  ... and {} more", functions.len() - max));
            }
        }

        lines.join("\n")
    }

    pub fn render_structure(&self, name: &str, structure: &StructureOffsets) -> String {
        let mut lines = Vec::new();

        let header = format!("struct {} {{ // size: {}, align: {}",
            name, structure.size, structure.alignment);

        if self.use_color {
            lines.push(header.cyan().bold().to_string());
        } else {
            lines.push(header);
        }

        let mut fields: Vec<_> = structure.fields.iter().collect();
        fields.sort_by_key(|(_, f)| f.offset);

        for (field_name, field) in fields {
            let field_line = format!("    /* +0x{:04x} */ {} {};",
                field.offset, field.field_type, field_name);

            if self.use_color {
                lines.push(format!("    {} {} {};",
                    format!("/* +0x{:04x} */", field.offset).dimmed(),
                    field.field_type.yellow(),
                    field_name.white()
                ));
            } else {
                lines.push(field_line);
            }
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    pub fn render_structure_list(&self, structures: &HashMap<String, StructureOffsets>) -> String {
        let mut lines = Vec::new();
        let mut sorted: Vec<_> = structures.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        if let Some(max) = self.max_items {
            sorted.truncate(max);
        }

        let header = format!("Structures ({})", structures.len());
        if self.use_color {
            lines.push(header.cyan().bold().to_string());
        } else {
            lines.push(header);
        }
        lines.push(String::new());

        for (name, structure) in sorted {
            lines.push(self.render_structure(name, structure));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn render_class(&self, class: &ClassOffset) -> String {
        let mut lines = Vec::new();

        let mut header = format!("class {}", class.name);
        if let Some(parent) = &class.parent {
            header.push_str(&format!(" : public {}", parent));
        }
        header.push_str(" {");

        if self.use_color {
            lines.push(header.cyan().bold().to_string());
        } else {
            lines.push(header);
        }

        if let Some(vtable) = class.vtable_address {
            let vtable_line = format!("    // VTable: 0x{:016x}", vtable);
            if self.use_color {
                lines.push(vtable_line.dimmed().to_string());
            } else {
                lines.push(vtable_line);
            }
        }

        let size_line = format!("    // Size: {} bytes", class.size);
        if self.use_color {
            lines.push(size_line.dimmed().to_string());
        } else {
            lines.push(size_line);
        }

        if !class.properties.is_empty() {
            lines.push(String::new());
            if self.use_color {
                lines.push("    // Properties:".dimmed().to_string());
            } else {
                lines.push("    // Properties:".to_string());
            }
            for prop in &class.properties {
                lines.push(format!("    //   {}", prop));
            }
        }

        if !class.methods.is_empty() {
            lines.push(String::new());
            if self.use_color {
                lines.push("    // Methods:".dimmed().to_string());
            } else {
                lines.push("    // Methods:".to_string());
            }
            for method in &class.methods {
                lines.push(format!("    //   {}", method));
            }
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    pub fn render_class_list(&self, classes: &[ClassOffset]) -> String {
        let mut lines = Vec::new();
        let mut sorted = classes.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(max) = self.max_items {
            sorted.truncate(max);
        }

        let header = format!("Classes ({})", classes.len());
        if self.use_color {
            lines.push(header.cyan().bold().to_string());
        } else {
            lines.push(header);
        }
        lines.push(String::new());

        for class in sorted {
            lines.push(self.render_class(&class));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn render_summary(&self, output: &OffsetOutput) -> String {
        let mut lines = Vec::new();

        let title = "Offset Generation Summary";
        if self.use_color {
            lines.push(format!("{}", "═".repeat(60).cyan()));
            lines.push(format!("{:^60}", title.cyan().bold()));
            lines.push(format!("{}", "═".repeat(60).cyan()));
        } else {
            lines.push("=".repeat(60));
            lines.push(format!("{:^60}", title));
            lines.push("=".repeat(60));
        }

        lines.push(String::new());

        let stats = vec![
            ("Target", output.target.name.clone()),
            ("Architecture", output.target.architecture.clone()),
            ("Platform", output.target.platform.clone()),
            ("Base Address", format!("0x{:x}", output.target.base_address)),
            ("Functions", output.functions.len().to_string()),
            ("Structures", output.structure_offsets.len().to_string()),
            ("Classes", output.classes.len().to_string()),
            ("Properties", output.properties.len().to_string()),
            ("Methods", output.methods.len().to_string()),
            ("Constants", output.constants.len().to_string()),
            ("Total Offsets", output.total_offsets().to_string()),
            ("Avg Confidence", format!("{:.1}%", output.statistics.average_confidence * 100.0)),
        ];

        for (key, value) in stats {
            if self.use_color {
                lines.push(format!("  {:<20} {}", key.cyan(), value.white()));
            } else {
                lines.push(format!("  {:<20} {}", key, value));
            }
        }

        lines.push(String::new());
        if self.use_color {
            lines.push(format!("{}", "─".repeat(60).dimmed()));
        } else {
            lines.push("-".repeat(60));
        }

        lines.join("\n")
    }

    pub fn render_full_output(&self, output: &OffsetOutput) -> String {
        let mut sections = Vec::new();

        sections.push(self.render_summary(output));
        sections.push(String::new());
        sections.push(self.render_function_list(&output.functions, output.target.base_address));
        sections.push(String::new());
        sections.push(self.render_structure_list(&output.structure_offsets));
        sections.push(String::new());
        sections.push(self.render_class_list(&output.classes));

        sections.join("\n")
    }

    pub fn render_diff_summary(&self, added: usize, removed: usize, changed: usize) -> String {
        let mut lines = Vec::new();

        if self.use_color {
            lines.push("Changes:".cyan().bold().to_string());
            lines.push(format!("  {} {}", format!("+{}", added).green(), "added".dimmed()));
            lines.push(format!("  {} {}", format!("-{}", removed).red(), "removed".dimmed()));
            lines.push(format!("  {} {}", format!("~{}", changed).yellow(), "changed".dimmed()));
        } else {
            lines.push("Changes:".to_string());
            lines.push(format!("  +{} added", added));
            lines.push(format!("  -{} removed", removed));
            lines.push(format!("  ~{} changed", changed));
        }

        lines.join("\n")
    }

    pub fn render_progress_bar(&self, current: usize, total: usize, width: usize) -> String {
        let progress = if total > 0 { current as f64 / total as f64 } else { 0.0 };
        let filled = (progress * width as f64) as usize;
        let empty = width - filled;

        let (fill_char, empty_char) = if self.use_unicode {
            ("█", "░")
        } else {
            ("#", "-")
        };

        let bar = format!("[{}{}]", fill_char.repeat(filled), empty_char.repeat(empty));
        let percent = format!("{:>5.1}%", progress * 100.0);

        if self.use_color {
            format!("{} {}", bar.cyan(), percent.green())
        } else {
            format!("{} {}", bar, percent)
        }
    }
}

impl Default for DisplayRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_renderer() -> DisplayRenderer {
    DisplayRenderer::new()
}

pub fn format_address(address: u64) -> String {
    format!("0x{:016x}", address)
}

pub fn format_address_short(address: u64) -> String {
    format!("0x{:x}", address)
}

pub fn format_confidence(confidence: f64) -> String {
    format!("{:.1}%", confidence * 100.0)
}

pub fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}
