// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct TemplateEngine {
    templates: HashMap<String, String>,
    variables: HashMap<String, String>,
    delimiters: (String, String),
    escape_html: bool,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            variables: HashMap::new(),
            delimiters: ("{{".to_string(), "}}".to_string()),
            escape_html: false,
        }
    }

    pub fn with_delimiters(mut self, open: &str, close: &str) -> Self {
        self.delimiters = (open.to_string(), close.to_string());
        self
    }

    pub fn with_html_escape(mut self, escape: bool) -> Self {
        self.escape_html = escape;
        self
    }

    pub fn load_template(&mut self, name: &str, template: &str) {
        self.templates.insert(name.to_string(), template.to_string());
    }

    pub fn load_template_file(&mut self, name: &str, path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.templates.insert(name.to_string(), content);
        Ok(())
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    pub fn set_variables(&mut self, vars: HashMap<String, String>) {
        self.variables.extend(vars);
    }

    pub fn set_from_output(&mut self, output: &OffsetOutput) {
        self.set_variable("version", &output.version);
        self.set_variable("generated_at", &output.generated_at);
        self.set_variable("target_name", &output.target.name);
        self.set_variable("target_arch", &output.target.architecture);
        self.set_variable("target_platform", &output.target.platform);
        self.set_variable("base_address", &format!("0x{:x}", output.target.base_address));
        self.set_variable("function_count", &output.functions.len().to_string());
        self.set_variable("structure_count", &output.structure_offsets.len().to_string());
        self.set_variable("class_count", &output.classes.len().to_string());
        self.set_variable("property_count", &output.properties.len().to_string());
        self.set_variable("method_count", &output.methods.len().to_string());
        self.set_variable("constant_count", &output.constants.len().to_string());
        self.set_variable("total_offsets", &output.total_offsets().to_string());
        self.set_variable("avg_confidence", &format!("{:.1}", output.statistics.average_confidence * 100.0));

        let functions_list = self.render_functions_list(&output.functions);
        self.set_variable("functions_list", &functions_list);

        let structures_list = self.render_structures_list(&output.structure_offsets);
        self.set_variable("structures_list", &structures_list);

        let classes_list = self.render_classes_list(&output.classes);
        self.set_variable("classes_list", &classes_list);
    }

    fn render_functions_list(&self, functions: &HashMap<String, FunctionOffset>) -> String {
        let mut sorted: Vec<_> = functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        sorted.iter()
            .map(|(name, func)| format!("{} = 0x{:x}", name, func.address))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_structures_list(&self, structures: &HashMap<String, StructureOffsets>) -> String {
        let mut sorted: Vec<_> = structures.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        let mut result = Vec::new();
        for (name, structure) in sorted {
            result.push(format!("{} (size: {})", name, structure.size));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                result.push(format!("  +0x{:04x} {}", field.offset, field_name));
            }
        }

        result.join("\n")
    }

    fn render_classes_list(&self, classes: &[ClassOffset]) -> String {
        let mut sorted = classes.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        sorted.iter()
            .map(|c| {
                let vtable_str = c.vtable_address
                    .map(|a| format!(" @ 0x{:x}", a))
                    .unwrap_or_default();
                format!("{}{}", c.name, vtable_str)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn render(&self, template_name: &str) -> Result<String, TemplateError> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| TemplateError::TemplateNotFound(template_name.to_string()))?;

        self.render_string(template)
    }

    pub fn render_string(&self, template: &str) -> Result<String, TemplateError> {
        let mut result = template.to_string();
        let (open, close) = &self.delimiters;

        for (key, value) in &self.variables {
            let placeholder = format!("{}{}{}", open, key, close);
            let replacement = if self.escape_html {
                Self::escape_html_chars(value)
            } else {
                value.clone()
            };
            result = result.replace(&placeholder, &replacement);
        }

        self.process_conditionals(&result)
    }

    fn process_conditionals(&self, input: &str) -> Result<String, TemplateError> {
        let mut result = input.to_string();
        let (open, close) = &self.delimiters;

        let if_pattern = format!("{}#if ", open);
        let endif_pattern = format!("{}#endif{}", open, close);

        while let Some(if_start) = result.find(&if_pattern) {
            let condition_end = result[if_start + if_pattern.len()..].find(close)
                .map(|i| if_start + if_pattern.len() + i)
                .ok_or_else(|| TemplateError::SyntaxError("Unclosed conditional".to_string()))?;

            let condition = &result[if_start + if_pattern.len()..condition_end];

            let endif_start = result[condition_end..].find(&endif_pattern)
                .map(|i| condition_end + i)
                .ok_or_else(|| TemplateError::SyntaxError("Missing #endif".to_string()))?;

            let content = &result[condition_end + close.len()..endif_start];

            let condition_met = self.evaluate_condition(condition);
            let replacement = if condition_met { content } else { "" };

            let full_block_end = endif_start + endif_pattern.len();
            result = format!("{}{}{}", &result[..if_start], replacement, &result[full_block_end..]);
        }

        Ok(result)
    }

    fn evaluate_condition(&self, condition: &str) -> bool {
        let parts: Vec<&str> = condition.split_whitespace().collect();

        if parts.len() == 1 {
            return self.variables.get(parts[0])
                .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                .unwrap_or(false);
        }

        if parts.len() == 3 {
            let left = self.variables.get(parts[0]).map(|s| s.as_str()).unwrap_or(parts[0]);
            let op = parts[1];
            let right = self.variables.get(parts[2]).map(|s| s.as_str()).unwrap_or(parts[2]);

            return match op {
                "==" => left == right,
                "!=" => left != right,
                ">" => left.parse::<i64>().ok()
                    .zip(right.parse::<i64>().ok())
                    .map(|(l, r)| l > r)
                    .unwrap_or(false),
                "<" => left.parse::<i64>().ok()
                    .zip(right.parse::<i64>().ok())
                    .map(|(l, r)| l < r)
                    .unwrap_or(false),
                ">=" => left.parse::<i64>().ok()
                    .zip(right.parse::<i64>().ok())
                    .map(|(l, r)| l >= r)
                    .unwrap_or(false),
                "<=" => left.parse::<i64>().ok()
                    .zip(right.parse::<i64>().ok())
                    .map(|(l, r)| l <= r)
                    .unwrap_or(false),
                _ => false,
            };
        }

        false
    }

    fn escape_html_chars(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    pub fn render_to_file(&self, template_name: &str, path: &Path) -> Result<(), TemplateError> {
        let content = self.render(template_name)?;
        fs::write(path, content).map_err(|e| TemplateError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn get_built_in_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();

        templates.insert("header_cpp".to_string(), r#"#pragma once

// Generated: {{generated_at}}
// Target: {{target_name}}
// Architecture: {{target_arch}}

#include <cstdint>

namespace Offsets {
    constexpr uintptr_t BASE_ADDRESS = {{base_address}};

{{functions_list}}
}
"#.to_string());

        templates.insert("summary_txt".to_string(), r#"Roblox Offset Generator Report
==============================

Target: {{target_name}}
Architecture: {{target_arch}}
Platform: {{target_platform}}
Generated: {{generated_at}}

Statistics:
-----------
Functions: {{function_count}}
Structures: {{structure_count}}
Classes: {{class_count}}
Properties: {{property_count}}
Methods: {{method_count}}
Constants: {{constant_count}}
Total Offsets: {{total_offsets}}
Average Confidence: {{avg_confidence}}%

{{#if function_count > 0}}
Functions:
----------
{{functions_list}}
{{#endif}}

{{#if structure_count > 0}}
Structures:
-----------
{{structures_list}}
{{#endif}}

{{#if class_count > 0}}
Classes:
--------
{{classes_list}}
{{#endif}}
"#.to_string());

        templates.insert("html_report".to_string(), r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Roblox Offsets - {{target_name}}</title>
    <style>
        body { font-family: monospace; background: #1a1a2e; color: #eee; padding: 20px; }
        h1 { color: #00d4ff; }
        .stat { color: #4ecdc4; }
        .address { color: #ff6b6b; }
    </style>
</head>
<body>
    <h1>Roblox Offset Report</h1>
    <p>Target: {{target_name}} | Arch: {{target_arch}} | Platform: {{target_platform}}</p>
    <p>Base Address: <span class="address">{{base_address}}</span></p>
    <p class="stat">Total Offsets: {{total_offsets}} | Confidence: {{avg_confidence}}%</p>
</body>
</html>
"#.to_string());

        templates
    }

    pub fn load_built_in_templates(&mut self) {
        let built_in = Self::get_built_in_templates();
        for (name, content) in built_in {
            self.load_template(&name, &content);
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum TemplateError {
    TemplateNotFound(String),
    SyntaxError(String),
    IoError(String),
    RenderError(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::TemplateNotFound(name) => write!(f, "Template not found: {}", name),
            TemplateError::SyntaxError(msg) => write!(f, "Template syntax error: {}", msg),
            TemplateError::IoError(msg) => write!(f, "IO error: {}", msg),
            TemplateError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

pub fn render_template(template: &str, variables: HashMap<String, String>) -> Result<String, TemplateError> {
    let mut engine = TemplateEngine::new();
    for (k, v) in variables {
        engine.set_variable(&k, &v);
    }
    engine.render_string(template)
}

pub fn render_output_template(output: &OffsetOutput, template: &str) -> Result<String, TemplateError> {
    let mut engine = TemplateEngine::new();
    engine.set_from_output(output);
    engine.render_string(template)
}
