// Tue Jan 13 2026 - Alex

pub mod json;
pub mod report;
pub mod manager;
pub mod formatter;
pub mod exporter;
pub mod template;
pub mod diff;
pub mod stats;

pub use json::JsonSerializer;
pub use report::ReportGenerator;
pub use manager::OutputManager;
pub use formatter::OutputFormatter;
pub use exporter::OffsetExporter;
pub use template::TemplateEngine;
pub use diff::DiffGenerator;
pub use stats::StatisticsCollector;

use crate::memory::Address;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetOutput {
    pub version: String,
    pub generated_at: String,
    pub target: TargetInfo,
    pub functions: HashMap<String, FunctionOffset>,
    pub structure_offsets: HashMap<String, StructureOffsets>,
    pub classes: Vec<ClassOffset>,
    pub properties: Vec<PropertyOffset>,
    pub methods: Vec<MethodOffset>,
    pub constants: Vec<ConstantOffset>,
    pub statistics: OutputStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub name: String,
    pub architecture: String,
    pub platform: String,
    pub version: Option<String>,
    pub hash: Option<String>,
    pub base_address: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionOffset {
    pub address: u64,
    pub confidence: f64,
    pub discovery_method: String,
    pub signature: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureOffsets {
    pub fields: HashMap<String, FieldOffset>,
    pub size: usize,
    pub alignment: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOffset {
    pub offset: usize,
    pub size: usize,
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassOffset {
    pub name: String,
    pub vtable_address: Option<u64>,
    pub size: usize,
    pub parent: Option<String>,
    pub properties: Vec<String>,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyOffset {
    pub name: String,
    pub class_name: String,
    pub getter: Option<u64>,
    pub setter: Option<u64>,
    pub offset: Option<usize>,
    pub property_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodOffset {
    pub name: String,
    pub class_name: String,
    pub address: u64,
    pub vtable_index: Option<usize>,
    pub is_virtual: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantOffset {
    pub name: String,
    pub address: u64,
    pub value: ConstantValue,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    String(String),
    Address(u64),
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputStatistics {
    pub total_functions: usize,
    pub total_structures: usize,
    pub total_classes: usize,
    pub total_properties: usize,
    pub total_methods: usize,
    pub total_constants: usize,
    pub scan_duration_ms: u64,
    pub memory_scanned_bytes: u64,
    pub patterns_matched: usize,
    pub symbols_resolved: usize,
    pub xrefs_analyzed: usize,
    pub average_confidence: f64,
}

impl OffsetOutput {
    pub fn new(target_name: &str) -> Self {
        Self {
            version: "1.0.0".to_string(),
            generated_at: chrono_now(),
            target: TargetInfo {
                name: target_name.to_string(),
                architecture: "arm64".to_string(),
                platform: "macos".to_string(),
                version: None,
                hash: None,
                base_address: 0x100000000,
            },
            functions: HashMap::new(),
            structure_offsets: HashMap::new(),
            classes: Vec::new(),
            properties: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
            statistics: OutputStatistics::default(),
        }
    }

    pub fn add_function(&mut self, name: &str, offset: FunctionOffset) {
        self.functions.insert(name.to_string(), offset);
        self.statistics.total_functions = self.functions.len();
    }

    pub fn add_structure(&mut self, name: &str, offsets: StructureOffsets) {
        self.structure_offsets.insert(name.to_string(), offsets);
        self.statistics.total_structures = self.structure_offsets.len();
    }

    pub fn add_class(&mut self, class: ClassOffset) {
        self.classes.push(class);
        self.statistics.total_classes = self.classes.len();
    }

    pub fn add_property(&mut self, property: PropertyOffset) {
        self.properties.push(property);
        self.statistics.total_properties = self.properties.len();
    }

    pub fn add_method(&mut self, method: MethodOffset) {
        self.methods.push(method);
        self.statistics.total_methods = self.methods.len();
    }

    pub fn add_constant(&mut self, constant: ConstantOffset) {
        self.constants.push(constant);
        self.statistics.total_constants = self.constants.len();
    }

    pub fn set_target_version(&mut self, version: &str) {
        self.target.version = Some(version.to_string());
    }

    pub fn set_target_hash(&mut self, hash: &str) {
        self.target.hash = Some(hash.to_string());
    }

    pub fn set_base_address(&mut self, addr: u64) {
        self.target.base_address = addr;
    }

    pub fn compute_statistics(&mut self) {
        self.statistics.total_functions = self.functions.len();
        self.statistics.total_structures = self.structure_offsets.len();
        self.statistics.total_classes = self.classes.len();
        self.statistics.total_properties = self.properties.len();
        self.statistics.total_methods = self.methods.len();
        self.statistics.total_constants = self.constants.len();

        let total_confidence: f64 = self.functions.values().map(|f| f.confidence).sum();
        if !self.functions.is_empty() {
            self.statistics.average_confidence = total_confidence / self.functions.len() as f64;
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionOffset> {
        self.functions.get(name)
    }

    pub fn get_structure(&self, name: &str) -> Option<&StructureOffsets> {
        self.structure_offsets.get(name)
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassOffset> {
        self.classes.iter().find(|c| c.name == name)
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn structure_count(&self) -> usize {
        self.structure_offsets.len()
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub fn total_offsets(&self) -> usize {
        self.functions.len() +
        self.structure_offsets.values().map(|s| s.fields.len()).sum::<usize>() +
        self.classes.len() +
        self.properties.len() +
        self.methods.len() +
        self.constants.len()
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}", secs)
}

impl FunctionOffset {
    pub fn new(address: u64, confidence: f64, method: &str) -> Self {
        Self {
            address,
            confidence,
            discovery_method: method.to_string(),
            signature: None,
            category: "unknown".to_string(),
        }
    }

    pub fn with_signature(mut self, sig: &str) -> Self {
        self.signature = Some(sig.to_string());
        self
    }

    pub fn with_category(mut self, cat: &str) -> Self {
        self.category = cat.to_string();
        self
    }
}

impl StructureOffsets {
    pub fn new(size: usize, alignment: usize) -> Self {
        Self {
            fields: HashMap::new(),
            size,
            alignment,
        }
    }

    pub fn add_field(&mut self, name: &str, offset: usize, size: usize, field_type: &str) {
        self.fields.insert(name.to_string(), FieldOffset {
            offset,
            size,
            field_type: field_type.to_string(),
        });
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldOffset> {
        self.fields.get(name)
    }
}

impl ClassOffset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vtable_address: None,
            size: 0,
            parent: None,
            properties: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn with_vtable(mut self, addr: u64) -> Self {
        self.vtable_address = Some(addr);
        self
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }
}
