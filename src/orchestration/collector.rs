// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::{FinderResult, FinderResults};
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct ResultCollector {
    functions: RwLock<HashMap<String, CollectedFunction>>,
    structure_offsets: RwLock<HashMap<String, HashMap<String, CollectedOffset>>>,
    classes: RwLock<HashMap<String, CollectedClass>>,
    properties: RwLock<HashMap<String, HashMap<String, CollectedProperty>>>,
    methods: RwLock<HashMap<String, HashMap<String, CollectedMethod>>>,
    constants: RwLock<HashMap<String, CollectedConstant>>,
}

impl ResultCollector {
    pub fn new() -> Self {
        Self {
            functions: RwLock::new(HashMap::new()),
            structure_offsets: RwLock::new(HashMap::new()),
            classes: RwLock::new(HashMap::new()),
            properties: RwLock::new(HashMap::new()),
            methods: RwLock::new(HashMap::new()),
            constants: RwLock::new(HashMap::new()),
        }
    }

    pub fn collect(&self, result: FinderResult, source: &str, confidence: f64) {
        match result.category.as_str() {
            "function" | "lua_api" => {
                self.collect_function(result.name, result.address, source, result.confidence);
            }
            "structure_offset" => {
                // Structure offsets need to be handled differently
                // This would need the structure name and field name from somewhere else
            }
            "class" => {
                self.collect_class(result.name, result.address, source, result.confidence);
            }
            "property" => {
                // Properties need class name and offset from somewhere else
            }
            "method" => {
                // Methods need class name from somewhere else
            }
            "constant" => {
                // Constants need value from somewhere else
            }
            _ => {
                // Default to function
                self.collect_function(result.name, result.address, source, result.confidence);
            }
        }
    }

    pub fn collect_results(&self, results: FinderResults, source: &str, confidence: f64) {
        for (name, addr) in results.functions {
            self.collect_function(name, addr, source, confidence);
        }

        for (struct_name, fields) in results.structure_offsets {
            for (field, offset) in fields {
                self.collect_structure_offset(struct_name.clone(), field, offset, source, confidence);
            }
        }

        for (name, addr) in results.classes {
            self.collect_class(name, addr, source, confidence);
        }

        for (class, props) in results.properties {
            for (prop, offset) in props {
                self.collect_property(class.clone(), prop, offset, source, confidence);
            }
        }

        for (class, methods) in results.methods {
            for (method, addr) in methods {
                self.collect_method(class.clone(), method, addr, source, confidence);
            }
        }

        for (name, value) in results.constants {
            self.collect_constant(name, value, source, confidence);
        }
    }

    fn collect_function(&self, name: String, addr: Address, source: &str, confidence: f64) {
        let mut funcs = self.functions.write();
        let entry = funcs.entry(name).or_insert_with(|| CollectedFunction {
            address: addr,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.address = addr;
            entry.confidence = confidence;
        }
    }

    fn collect_structure_offset(&self, struct_name: String, field: String, offset: u64, source: &str, confidence: f64) {
        let mut offsets = self.structure_offsets.write();
        let struct_entry = offsets.entry(struct_name).or_default();
        let entry = struct_entry.entry(field).or_insert_with(|| CollectedOffset {
            offset,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.offset = offset;
            entry.confidence = confidence;
        }
    }

    fn collect_class(&self, name: String, addr: Address, source: &str, confidence: f64) {
        let mut classes = self.classes.write();
        let entry = classes.entry(name).or_insert_with(|| CollectedClass {
            address: addr,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.address = addr;
            entry.confidence = confidence;
        }
    }

    fn collect_property(&self, class: String, prop: String, offset: u64, source: &str, confidence: f64) {
        let mut props = self.properties.write();
        let class_entry = props.entry(class).or_default();
        let entry = class_entry.entry(prop).or_insert_with(|| CollectedProperty {
            offset,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.offset = offset;
            entry.confidence = confidence;
        }
    }

    fn collect_method(&self, class: String, method: String, addr: Address, source: &str, confidence: f64) {
        let mut methods = self.methods.write();
        let class_entry = methods.entry(class).or_default();
        let entry = class_entry.entry(method).or_insert_with(|| CollectedMethod {
            address: addr,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.address = addr;
            entry.confidence = confidence;
        }
    }

    fn collect_constant(&self, name: String, value: u64, source: &str, confidence: f64) {
        let mut consts = self.constants.write();
        let entry = consts.entry(name).or_insert_with(|| CollectedConstant {
            value,
            sources: Vec::new(),
            confidence: 0.0,
        });

        entry.sources.push(source.to_string());
        if confidence > entry.confidence {
            entry.value = value;
            entry.confidence = confidence;
        }
    }

    pub fn pattern_count(&self) -> usize {
        self.functions.read().len()
    }

    pub fn symbol_count(&self) -> usize {
        self.classes.read().len()
    }

    pub fn xref_count(&self) -> usize {
        self.structure_offsets.read().values()
            .map(|m| m.len())
            .sum()
    }

    pub fn structure_count(&self) -> usize {
        self.structure_offsets.read().len()
    }

    pub fn total_count(&self) -> usize {
        self.functions.read().len()
            + self.structure_offsets.read().values().map(|m| m.len()).sum::<usize>()
            + self.classes.read().len()
            + self.properties.read().values().map(|m| m.len()).sum::<usize>()
            + self.methods.read().values().map(|m| m.len()).sum::<usize>()
            + self.constants.read().len()
    }

    pub fn to_finder_results(&self) -> FinderResults {
        let mut results = FinderResults::new();

        for (name, func) in self.functions.read().iter() {
            results.functions.insert(name.clone(), func.address);
        }

        for (struct_name, fields) in self.structure_offsets.read().iter() {
            for (field, offset) in fields {
                results.structure_offsets
                    .entry(struct_name.clone())
                    .or_default()
                    .insert(field.clone(), offset.offset);
            }
        }

        for (name, class) in self.classes.read().iter() {
            results.classes.insert(name.clone(), class.address);
        }

        for (class, props) in self.properties.read().iter() {
            for (prop, p) in props {
                results.properties
                    .entry(class.clone())
                    .or_default()
                    .insert(prop.clone(), p.offset);
            }
        }

        for (class, methods) in self.methods.read().iter() {
            for (method, m) in methods {
                results.methods
                    .entry(class.clone())
                    .or_default()
                    .insert(method.clone(), m.address);
            }
        }

        for (name, constant) in self.constants.read().iter() {
            results.constants.insert(name.clone(), constant.value);
        }

        results
    }

    pub fn clear(&self) {
        self.functions.write().clear();
        self.structure_offsets.write().clear();
        self.classes.write().clear();
        self.properties.write().clear();
        self.methods.write().clear();
        self.constants.write().clear();
    }
}

impl Default for ResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CollectedFunction {
    pub address: Address,
    pub sources: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct CollectedOffset {
    pub offset: u64,
    pub sources: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct CollectedClass {
    pub address: Address,
    pub sources: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct CollectedProperty {
    pub offset: u64,
    pub sources: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct CollectedMethod {
    pub address: Address,
    pub sources: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct CollectedConstant {
    pub value: u64,
    pub sources: Vec<String>,
    pub confidence: f64,
}
