// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, PropertyOffset, MethodOffset, ConstantOffset, OutputStatistics};
use serde_json::{Value, json, to_string, to_string_pretty};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read, BufWriter};
use std::path::Path;

pub struct JsonSerializer {
    pretty_print: bool,
    indent_size: usize,
    sort_keys: bool,
    include_metadata: bool,
    include_statistics: bool,
}

impl JsonSerializer {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
            indent_size: 2,
            sort_keys: true,
            include_metadata: true,
            include_statistics: true,
        }
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    pub fn with_sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }

    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn with_statistics(mut self, include: bool) -> Self {
        self.include_statistics = include;
        self
    }

    pub fn serialize(&self, output: &OffsetOutput) -> Result<String, JsonError> {
        let value = self.build_json_value(output)?;

        if self.pretty_print {
            to_string_pretty(&value).map_err(|e| JsonError::SerializationError(e.to_string()))
        } else {
            to_string(&value).map_err(|e| JsonError::SerializationError(e.to_string()))
        }
    }

    pub fn serialize_to_file<P: AsRef<Path>>(&self, output: &OffsetOutput, path: P) -> Result<(), JsonError> {
        let json_str = self.serialize(output)?;

        let file = File::create(path.as_ref())
            .map_err(|e| JsonError::IoError(e.to_string()))?;

        let mut writer = BufWriter::new(file);
        writer.write_all(json_str.as_bytes())
            .map_err(|e| JsonError::IoError(e.to_string()))?;

        Ok(())
    }

    fn build_json_value(&self, output: &OffsetOutput) -> Result<Value, JsonError> {
        let mut root = serde_json::Map::new();

        if self.include_metadata {
            root.insert("version".to_string(), json!(output.version));
            root.insert("generated_at".to_string(), json!(output.generated_at));
            root.insert("target".to_string(), self.serialize_target(&output.target)?);
        }

        root.insert("functions".to_string(), self.serialize_functions(&output.functions)?);
        root.insert("structure_offsets".to_string(), self.serialize_structures(&output.structure_offsets)?);
        root.insert("classes".to_string(), self.serialize_classes(&output.classes)?);
        root.insert("properties".to_string(), self.serialize_properties(&output.properties)?);
        root.insert("methods".to_string(), self.serialize_methods(&output.methods)?);
        root.insert("constants".to_string(), self.serialize_constants(&output.constants)?);

        if self.include_statistics {
            root.insert("statistics".to_string(), self.serialize_statistics(&output.statistics)?);
        }

        Ok(Value::Object(root))
    }

    fn serialize_target(&self, target: &crate::output::TargetInfo) -> Result<Value, JsonError> {
        Ok(json!({
            "name": target.name,
            "architecture": target.architecture,
            "platform": target.platform,
            "version": target.version,
            "hash": target.hash,
            "base_address": format!("0x{:x}", target.base_address)
        }))
    }

    fn serialize_functions(&self, functions: &HashMap<String, FunctionOffset>) -> Result<Value, JsonError> {
        let mut map = serde_json::Map::new();

        let mut sorted_keys: Vec<_> = functions.keys().collect();
        if self.sort_keys {
            sorted_keys.sort();
        }

        for key in sorted_keys {
            let func = &functions[key];
            map.insert(key.clone(), json!({
                "address": format!("0x{:x}", func.address),
                "confidence": func.confidence,
                "discovery_method": func.discovery_method,
                "signature": func.signature,
                "category": func.category
            }));
        }

        Ok(Value::Object(map))
    }

    fn serialize_structures(&self, structures: &HashMap<String, StructureOffsets>) -> Result<Value, JsonError> {
        let mut map = serde_json::Map::new();

        let mut sorted_keys: Vec<_> = structures.keys().collect();
        if self.sort_keys {
            sorted_keys.sort();
        }

        for key in sorted_keys {
            let structure = &structures[key];
            let mut fields_map = serde_json::Map::new();

            let mut field_keys: Vec<_> = structure.fields.keys().collect();
            if self.sort_keys {
                field_keys.sort();
            }

            for field_key in field_keys {
                let field = &structure.fields[field_key];
                fields_map.insert(field_key.clone(), json!({
                    "offset": format!("0x{:x}", field.offset),
                    "size": field.size,
                    "type": field.field_type
                }));
            }

            map.insert(key.clone(), json!({
                "fields": Value::Object(fields_map),
                "size": structure.size,
                "alignment": structure.alignment
            }));
        }

        Ok(Value::Object(map))
    }

    fn serialize_classes(&self, classes: &[ClassOffset]) -> Result<Value, JsonError> {
        let mut arr = Vec::new();

        for class in classes {
            arr.push(json!({
                "name": class.name,
                "vtable_address": class.vtable_address.map(|a| format!("0x{:x}", a)),
                "size": class.size,
                "parent": class.parent,
                "properties": class.properties,
                "methods": class.methods
            }));
        }

        Ok(Value::Array(arr))
    }

    fn serialize_properties(&self, properties: &[PropertyOffset]) -> Result<Value, JsonError> {
        let mut arr = Vec::new();

        for prop in properties {
            arr.push(json!({
                "name": prop.name,
                "class_name": prop.class_name,
                "getter": prop.getter.map(|a| format!("0x{:x}", a)),
                "setter": prop.setter.map(|a| format!("0x{:x}", a)),
                "offset": prop.offset.map(|o| format!("0x{:x}", o)),
                "type": prop.property_type
            }));
        }

        Ok(Value::Array(arr))
    }

    fn serialize_methods(&self, methods: &[MethodOffset]) -> Result<Value, JsonError> {
        let mut arr = Vec::new();

        for method in methods {
            arr.push(json!({
                "name": method.name,
                "class_name": method.class_name,
                "address": format!("0x{:x}", method.address),
                "vtable_index": method.vtable_index,
                "is_virtual": method.is_virtual,
                "signature": method.signature
            }));
        }

        Ok(Value::Array(arr))
    }

    fn serialize_constants(&self, constants: &[ConstantOffset]) -> Result<Value, JsonError> {
        let mut arr = Vec::new();

        for constant in constants {
            let value = match &constant.value {
                crate::output::ConstantValue::Integer(i) => json!(i),
                crate::output::ConstantValue::Float(f) => json!(f),
                crate::output::ConstantValue::String(s) => json!(s),
                crate::output::ConstantValue::Address(a) => json!(format!("0x{:x}", a)),
                crate::output::ConstantValue::Unknown => json!(null),
            };

            arr.push(json!({
                "name": constant.name,
                "address": format!("0x{:x}", constant.address),
                "value": value,
                "category": constant.category
            }));
        }

        Ok(Value::Array(arr))
    }

    fn serialize_statistics(&self, stats: &OutputStatistics) -> Result<Value, JsonError> {
        Ok(json!({
            "total_functions": stats.total_functions,
            "total_structures": stats.total_structures,
            "total_classes": stats.total_classes,
            "total_properties": stats.total_properties,
            "total_methods": stats.total_methods,
            "total_constants": stats.total_constants,
            "scan_duration_ms": stats.scan_duration_ms,
            "memory_scanned_bytes": stats.memory_scanned_bytes,
            "patterns_matched": stats.patterns_matched,
            "symbols_resolved": stats.symbols_resolved,
            "xrefs_analyzed": stats.xrefs_analyzed,
            "average_confidence": stats.average_confidence
        }))
    }

    pub fn deserialize(&self, json_str: &str) -> Result<OffsetOutput, JsonError> {
        serde_json::from_str(json_str)
            .map_err(|e| JsonError::DeserializationError(e.to_string()))
    }

    pub fn deserialize_from_file<P: AsRef<Path>>(&self, path: P) -> Result<OffsetOutput, JsonError> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| JsonError::IoError(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| JsonError::IoError(e.to_string()))?;

        self.deserialize(&contents)
    }

    pub fn merge(&self, base: &OffsetOutput, overlay: &OffsetOutput) -> OffsetOutput {
        let mut merged = base.clone();

        for (name, func) in &overlay.functions {
            if let Some(existing) = merged.functions.get(name) {
                if func.confidence > existing.confidence {
                    merged.functions.insert(name.clone(), func.clone());
                }
            } else {
                merged.functions.insert(name.clone(), func.clone());
            }
        }

        for (name, structure) in &overlay.structure_offsets {
            merged.structure_offsets.insert(name.clone(), structure.clone());
        }

        for class in &overlay.classes {
            if !merged.classes.iter().any(|c| c.name == class.name) {
                merged.classes.push(class.clone());
            }
        }

        merged.compute_statistics();
        merged
    }

    pub fn diff(&self, old: &OffsetOutput, new: &OffsetOutput) -> JsonDiff {
        let mut added_functions = Vec::new();
        let mut removed_functions = Vec::new();
        let mut changed_functions = Vec::new();

        for (name, func) in &new.functions {
            if let Some(old_func) = old.functions.get(name) {
                if old_func.address != func.address {
                    changed_functions.push((name.clone(), old_func.address, func.address));
                }
            } else {
                added_functions.push(name.clone());
            }
        }

        for name in old.functions.keys() {
            if !new.functions.contains_key(name) {
                removed_functions.push(name.clone());
            }
        }

        JsonDiff {
            added_functions,
            removed_functions,
            changed_functions,
            added_structures: Vec::new(),
            removed_structures: Vec::new(),
            changed_structures: Vec::new(),
        }
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsonDiff {
    pub added_functions: Vec<String>,
    pub removed_functions: Vec<String>,
    pub changed_functions: Vec<(String, u64, u64)>,
    pub added_structures: Vec<String>,
    pub removed_structures: Vec<String>,
    pub changed_structures: Vec<String>,
}

impl JsonDiff {
    pub fn has_changes(&self) -> bool {
        !self.added_functions.is_empty() ||
        !self.removed_functions.is_empty() ||
        !self.changed_functions.is_empty() ||
        !self.added_structures.is_empty() ||
        !self.removed_structures.is_empty() ||
        !self.changed_structures.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Added: {} functions, {} structures\nRemoved: {} functions, {} structures\nChanged: {} functions, {} structures",
            self.added_functions.len(),
            self.added_structures.len(),
            self.removed_functions.len(),
            self.removed_structures.len(),
            self.changed_functions.len(),
            self.changed_structures.len()
        )
    }
}

#[derive(Debug, Clone)]
pub enum JsonError {
    SerializationError(String),
    DeserializationError(String),
    IoError(String),
    ValidationError(String),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            JsonError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            JsonError::IoError(e) => write!(f, "IO error: {}", e),
            JsonError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for JsonError {}

pub fn to_json_string(output: &OffsetOutput) -> Result<String, JsonError> {
    JsonSerializer::new().serialize(output)
}

pub fn to_json_file<P: AsRef<Path>>(output: &OffsetOutput, path: P) -> Result<(), JsonError> {
    JsonSerializer::new().serialize_to_file(output, path)
}

pub fn from_json_string(json_str: &str) -> Result<OffsetOutput, JsonError> {
    JsonSerializer::new().deserialize(json_str)
}

pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<OffsetOutput, JsonError> {
    JsonSerializer::new().deserialize_from_file(path)
}
