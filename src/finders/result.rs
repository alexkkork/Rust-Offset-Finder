// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinderResult {
    pub name: String,
    pub address: Address,
    pub confidence: f64,
    pub method: String,
    pub category: String,
    pub signature: Option<String>,
}

impl FinderResult {
    pub fn new(name: String, address: Address, confidence: f64) -> Self {
        Self {
            name,
            address,
            confidence,
            method: "unknown".to_string(),
            category: "unknown".to_string(),
            signature: None,
        }
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }

    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.85
    }

    pub fn is_medium_confidence(&self) -> bool {
        self.confidence >= 0.65 && self.confidence < 0.85
    }

    pub fn is_low_confidence(&self) -> bool {
        self.confidence < 0.65
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureOffsetResult {
    pub structure_name: String,
    pub field_name: String,
    pub offset: u64,
    pub size: Option<u64>,
    pub confidence: f64,
    pub method: String,
}

impl StructureOffsetResult {
    pub fn new(structure_name: String, field_name: String, offset: u64) -> Self {
        Self {
            structure_name,
            field_name,
            offset,
            size: None,
            confidence: 0.5,
            method: "unknown".to_string(),
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassResult {
    pub name: String,
    pub address: Address,
    pub vtable_address: Option<Address>,
    pub size: Option<u64>,
    pub parent_class: Option<String>,
    pub confidence: f64,
}

impl ClassResult {
    pub fn new(name: String, address: Address) -> Self {
        Self {
            name,
            address,
            vtable_address: None,
            size: None,
            parent_class: None,
            confidence: 0.5,
        }
    }

    pub fn with_vtable(mut self, vtable: Address) -> Self {
        self.vtable_address = Some(vtable);
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent_class = Some(parent.to_string());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyResult {
    pub class_name: String,
    pub property_name: String,
    pub getter_address: Option<Address>,
    pub setter_address: Option<Address>,
    pub offset: Option<u64>,
    pub property_type: Option<String>,
    pub confidence: f64,
}

impl PropertyResult {
    pub fn new(class_name: String, property_name: String) -> Self {
        Self {
            class_name,
            property_name,
            getter_address: None,
            setter_address: None,
            offset: None,
            property_type: None,
            confidence: 0.5,
        }
    }

    pub fn with_getter(mut self, getter: Address) -> Self {
        self.getter_address = Some(getter);
        self
    }

    pub fn with_setter(mut self, setter: Address) -> Self {
        self.setter_address = Some(setter);
        self
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_type(mut self, prop_type: &str) -> Self {
        self.property_type = Some(prop_type.to_string());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodResult {
    pub class_name: String,
    pub method_name: String,
    pub address: Address,
    pub vtable_index: Option<u32>,
    pub signature: Option<String>,
    pub is_virtual: bool,
    pub confidence: f64,
}

impl MethodResult {
    pub fn new(class_name: String, method_name: String, address: Address) -> Self {
        Self {
            class_name,
            method_name,
            address,
            vtable_index: None,
            signature: None,
            is_virtual: false,
            confidence: 0.5,
        }
    }

    pub fn with_vtable_index(mut self, index: u32) -> Self {
        self.vtable_index = Some(index);
        self.is_virtual = true;
        self
    }

    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }

    pub fn set_virtual(mut self, is_virtual: bool) -> Self {
        self.is_virtual = is_virtual;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantResult {
    pub name: String,
    pub address: Address,
    pub value: ConstantValue,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    String(String),
    Pointer(Address),
    Unknown,
}

impl ConstantResult {
    pub fn new(name: String, address: Address, value: ConstantValue) -> Self {
        Self {
            name,
            address,
            value,
            confidence: 0.5,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombinedResults {
    pub functions: Vec<FinderResult>,
    pub structure_offsets: Vec<StructureOffsetResult>,
    pub classes: Vec<ClassResult>,
    pub properties: Vec<PropertyResult>,
    pub methods: Vec<MethodResult>,
    pub constants: Vec<ConstantResult>,
}

impl CombinedResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_function(&mut self, result: FinderResult) {
        self.functions.push(result);
    }

    pub fn add_structure_offset(&mut self, result: StructureOffsetResult) {
        self.structure_offsets.push(result);
    }

    pub fn add_class(&mut self, result: ClassResult) {
        self.classes.push(result);
    }

    pub fn add_property(&mut self, result: PropertyResult) {
        self.properties.push(result);
    }

    pub fn add_method(&mut self, result: MethodResult) {
        self.methods.push(result);
    }

    pub fn add_constant(&mut self, result: ConstantResult) {
        self.constants.push(result);
    }

    pub fn total_count(&self) -> usize {
        self.functions.len()
            + self.structure_offsets.len()
            + self.classes.len()
            + self.properties.len()
            + self.methods.len()
            + self.constants.len()
    }

    pub fn high_confidence_count(&self) -> usize {
        self.functions.iter().filter(|f| f.is_high_confidence()).count()
            + self.structure_offsets.iter().filter(|s| s.confidence >= 0.85).count()
            + self.classes.iter().filter(|c| c.confidence >= 0.85).count()
            + self.properties.iter().filter(|p| p.confidence >= 0.85).count()
            + self.methods.iter().filter(|m| m.confidence >= 0.85).count()
            + self.constants.iter().filter(|c| c.confidence >= 0.85).count()
    }

    pub fn merge(&mut self, other: CombinedResults) {
        self.functions.extend(other.functions);
        self.structure_offsets.extend(other.structure_offsets);
        self.classes.extend(other.classes);
        self.properties.extend(other.properties);
        self.methods.extend(other.methods);
        self.constants.extend(other.constants);
    }

    pub fn to_json_map(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();

        let mut functions_map = HashMap::new();
        for func in &self.functions {
            functions_map.insert(func.name.clone(), serde_json::json!({
                "address": format!("0x{:x}", func.address.as_u64()),
                "confidence": func.confidence,
                "method": func.method,
                "category": func.category,
                "signature": func.signature,
            }));
        }
        map.insert("functions".to_string(), serde_json::to_value(functions_map).unwrap());

        let mut structure_offsets_map: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
        for offset in &self.structure_offsets {
            let entry = structure_offsets_map.entry(offset.structure_name.clone()).or_default();
            entry.insert(offset.field_name.clone(), serde_json::json!({
                "offset": format!("0x{:x}", offset.offset),
                "size": offset.size,
                "confidence": offset.confidence,
                "method": offset.method,
            }));
        }
        map.insert("structure_offsets".to_string(), serde_json::to_value(structure_offsets_map).unwrap());

        let mut classes_map = HashMap::new();
        for class in &self.classes {
            classes_map.insert(class.name.clone(), serde_json::json!({
                "address": format!("0x{:x}", class.address.as_u64()),
                "vtable": class.vtable_address.map(|v| format!("0x{:x}", v.as_u64())),
                "size": class.size,
                "parent": class.parent_class,
                "confidence": class.confidence,
            }));
        }
        map.insert("classes".to_string(), serde_json::to_value(classes_map).unwrap());

        let mut properties_map: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        for prop in &self.properties {
            let entry = properties_map.entry(prop.class_name.clone()).or_default();
            entry.push(serde_json::json!({
                "name": prop.property_name,
                "getter": prop.getter_address.map(|a| format!("0x{:x}", a.as_u64())),
                "setter": prop.setter_address.map(|a| format!("0x{:x}", a.as_u64())),
                "offset": prop.offset.map(|o| format!("0x{:x}", o)),
                "type": prop.property_type,
                "confidence": prop.confidence,
            }));
        }
        map.insert("properties".to_string(), serde_json::to_value(properties_map).unwrap());

        let mut methods_map: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        for method in &self.methods {
            let entry = methods_map.entry(method.class_name.clone()).or_default();
            entry.push(serde_json::json!({
                "name": method.method_name,
                "address": format!("0x{:x}", method.address.as_u64()),
                "vtable_index": method.vtable_index,
                "signature": method.signature,
                "is_virtual": method.is_virtual,
                "confidence": method.confidence,
            }));
        }
        map.insert("methods".to_string(), serde_json::to_value(methods_map).unwrap());

        let mut constants_map = HashMap::new();
        for constant in &self.constants {
            let value_repr = match &constant.value {
                ConstantValue::Integer(i) => serde_json::json!(i),
                ConstantValue::Float(f) => serde_json::json!(f),
                ConstantValue::String(s) => serde_json::json!(s),
                ConstantValue::Pointer(p) => serde_json::json!(format!("0x{:x}", p.as_u64())),
                ConstantValue::Unknown => serde_json::json!(null),
            };
            constants_map.insert(constant.name.clone(), serde_json::json!({
                "address": format!("0x{:x}", constant.address.as_u64()),
                "value": value_repr,
                "confidence": constant.confidence,
            }));
        }
        map.insert("constants".to_string(), serde_json::to_value(constants_map).unwrap());

        map
    }
}
