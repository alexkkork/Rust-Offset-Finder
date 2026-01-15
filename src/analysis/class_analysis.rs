// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::{VTableInfo, VTableAnalyzer, StringAnalyzer, ReferenceAnalyzer};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ClassAnalyzer {
    reader: Arc<dyn MemoryReader>,
    vtable_analyzer: VTableAnalyzer,
    string_analyzer: StringAnalyzer,
    classes: HashMap<String, ClassInfo>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub vtable_address: Option<Address>,
    pub size: usize,
    pub alignment: usize,
    pub members: Vec<ClassMember>,
    pub methods: Vec<ClassMethod>,
    pub base_classes: Vec<BaseClass>,
    pub derived_classes: Vec<String>,
    pub constructor: Option<Address>,
    pub destructor: Option<Address>,
    pub type_info: Option<Address>,
    pub instances: Vec<Address>,
}

#[derive(Debug, Clone)]
pub struct ClassMember {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub member_type: MemberType,
    pub is_pointer: bool,
}

#[derive(Debug, Clone)]
pub enum MemberType {
    Primitive(PrimitiveKind),
    Pointer(Box<MemberType>),
    Array(Box<MemberType>, usize),
    Class(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveKind {
    Bool,
    Char,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
}

#[derive(Debug, Clone)]
pub struct ClassMethod {
    pub name: String,
    pub address: Address,
    pub vtable_index: Option<usize>,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_static: bool,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct BaseClass {
    pub name: String,
    pub offset: usize,
    pub vtable_offset: usize,
    pub is_virtual: bool,
}

impl ClassAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            vtable_analyzer: VTableAnalyzer::new(reader.clone()),
            string_analyzer: StringAnalyzer::new(reader.clone()),
            classes: HashMap::new(),
        }
    }

    pub fn analyze_from_vtable(&mut self, vtable_addr: Address) -> Result<Option<ClassInfo>, MemoryError> {
        let vtable_info = match self.vtable_analyzer.analyze(vtable_addr)? {
            Some(v) => v,
            None => return Ok(None),
        };

        let name = vtable_info.class_name.clone().unwrap_or_else(|| {
            format!("Class_{:016x}", vtable_addr.as_u64())
        });

        if self.classes.contains_key(&name) {
            return Ok(self.classes.get(&name).cloned());
        }

        let mut methods = Vec::new();
        for entry in &vtable_info.entries {
            methods.push(ClassMethod {
                name: entry.function_name.clone().unwrap_or_else(|| {
                    format!("vfunc_{}", entry.index)
                }),
                address: entry.target,
                vtable_index: Some(entry.index),
                is_virtual: true,
                is_pure_virtual: entry.is_pure_virtual,
                is_static: false,
                is_const: false,
            });
        }

        let constructor = self.find_constructor(&vtable_info)?;
        let destructor = self.find_destructor(&vtable_info)?;

        let class_info = ClassInfo {
            name: name.clone(),
            vtable_address: Some(vtable_addr),
            size: 0,
            alignment: 8,
            members: Vec::new(),
            methods,
            base_classes: Vec::new(),
            derived_classes: Vec::new(),
            constructor,
            destructor,
            type_info: vtable_info.type_info_ptr,
            instances: Vec::new(),
        };

        self.classes.insert(name.clone(), class_info.clone());
        Ok(Some(class_info))
    }

    fn find_constructor(&self, _vtable_info: &VTableInfo) -> Result<Option<Address>, MemoryError> {
        Ok(None)
    }

    fn find_destructor(&self, vtable_info: &VTableInfo) -> Result<Option<Address>, MemoryError> {
        if let Some(entry) = vtable_info.entries.first() {
            let target_addr = entry.target;
            if let Ok(insn) = self.reader.read_u32(target_addr) {
                if (insn & 0xFFFFFFFF) == 0xD503233F {
                    return Ok(Some(target_addr));
                }
            }
        }
        Ok(None)
    }

    pub fn analyze_instance(&mut self, instance_addr: Address) -> Result<Option<String>, MemoryError> {
        let vtable_ptr = self.reader.read_u64(instance_addr)?;

        if vtable_ptr < 0x100000000 || vtable_ptr >= 0x800000000000 {
            return Ok(None);
        }

        let vtable_addr = Address::new(vtable_ptr);

        if let Some(class_info) = self.analyze_from_vtable(vtable_addr)? {
            let class_name = class_info.name.clone();

            if let Some(info) = self.classes.get_mut(&class_name) {
                if !info.instances.contains(&instance_addr) {
                    info.instances.push(instance_addr);
                }
            }

            return Ok(Some(class_name));
        }

        Ok(None)
    }

    pub fn infer_members(&mut self, class_name: &str, instance_addr: Address, size: usize) -> Result<Vec<ClassMember>, MemoryError> {
        let mut members = Vec::new();
        let data = self.reader.read_bytes(instance_addr, size)?;

        let mut offset = 8;
        while offset < size {
            let remaining = size - offset;
            let member_addr = instance_addr + offset as u64;

            if remaining >= 8 {
                let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());

                if val >= 0x100000000 && val < 0x800000000000 {
                    members.push(ClassMember {
                        name: format!("field_{:x}", offset),
                        offset,
                        size: 8,
                        member_type: MemberType::Pointer(Box::new(MemberType::Unknown)),
                        is_pointer: true,
                    });
                    offset += 8;
                    continue;
                }
            }

            if remaining >= 8 {
                let val = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                let f64_val = f64::from_bits(val);

                if f64_val.is_finite() && f64_val.abs() < 1e10 && f64_val != 0.0 {
                    members.push(ClassMember {
                        name: format!("field_{:x}", offset),
                        offset,
                        size: 8,
                        member_type: MemberType::Primitive(PrimitiveKind::Double),
                        is_pointer: false,
                    });
                    offset += 8;
                    continue;
                }
            }

            if remaining >= 4 {
                let val = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                let f32_val = f32::from_bits(val);

                if f32_val.is_finite() && f32_val.abs() < 1e10 && f32_val != 0.0 {
                    members.push(ClassMember {
                        name: format!("field_{:x}", offset),
                        offset,
                        size: 4,
                        member_type: MemberType::Primitive(PrimitiveKind::Float),
                        is_pointer: false,
                    });
                    offset += 4;
                    continue;
                }

                members.push(ClassMember {
                    name: format!("field_{:x}", offset),
                    offset,
                    size: 4,
                    member_type: MemberType::Primitive(PrimitiveKind::Int32),
                    is_pointer: false,
                });
                offset += 4;
                continue;
            }

            members.push(ClassMember {
                name: format!("field_{:x}", offset),
                offset,
                size: 1,
                member_type: MemberType::Primitive(PrimitiveKind::UInt8),
                is_pointer: false,
            });
            offset += 1;
        }

        if let Some(class_info) = self.classes.get_mut(class_name) {
            class_info.members = members.clone();
            class_info.size = size;
        }

        Ok(members)
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    pub fn get_all_classes(&self) -> Vec<&ClassInfo> {
        self.classes.values().collect()
    }

    pub fn find_class_by_vtable(&self, vtable_addr: Address) -> Option<&ClassInfo> {
        self.classes.values().find(|c| c.vtable_address == Some(vtable_addr))
    }

    pub fn build_class_hierarchy(&mut self) -> ClassHierarchy {
        let mut hierarchy = ClassHierarchy::new();

        for class in self.classes.values() {
            hierarchy.add_class(class.name.clone());

            for base in &class.base_classes {
                hierarchy.add_inheritance(&class.name, &base.name);
            }
        }

        hierarchy
    }

    pub fn clear(&mut self) {
        self.classes.clear();
        self.vtable_analyzer.clear();
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }
}

#[derive(Debug, Clone)]
pub struct ClassHierarchy {
    classes: HashMap<String, HierarchyNode>,
}

#[derive(Debug, Clone)]
struct HierarchyNode {
    name: String,
    bases: Vec<String>,
    derived: Vec<String>,
}

impl ClassHierarchy {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn add_class(&mut self, name: String) {
        self.classes.entry(name.clone()).or_insert(HierarchyNode {
            name,
            bases: Vec::new(),
            derived: Vec::new(),
        });
    }

    pub fn add_inheritance(&mut self, derived: &str, base: &str) {
        if let Some(derived_node) = self.classes.get_mut(derived) {
            if !derived_node.bases.contains(&base.to_string()) {
                derived_node.bases.push(base.to_string());
            }
        }

        self.add_class(base.to_string());

        if let Some(base_node) = self.classes.get_mut(base) {
            if !base_node.derived.contains(&derived.to_string()) {
                base_node.derived.push(derived.to_string());
            }
        }
    }

    pub fn get_bases(&self, class: &str) -> Vec<&str> {
        self.classes.get(class)
            .map(|n| n.bases.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn get_derived(&self, class: &str) -> Vec<&str> {
        self.classes.get(class)
            .map(|n| n.derived.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn get_root_classes(&self) -> Vec<&str> {
        self.classes.values()
            .filter(|n| n.bases.is_empty())
            .map(|n| n.name.as_str())
            .collect()
    }

    pub fn get_leaf_classes(&self) -> Vec<&str> {
        self.classes.values()
            .filter(|n| n.derived.is_empty())
            .map(|n| n.name.as_str())
            .collect()
    }

    pub fn depth(&self, class: &str) -> usize {
        let mut max_depth = 0;
        if let Some(node) = self.classes.get(class) {
            for base in &node.bases {
                max_depth = max_depth.max(1 + self.depth(base));
            }
        }
        max_depth
    }

    pub fn is_base_of(&self, base: &str, derived: &str) -> bool {
        if base == derived {
            return false;
        }

        if let Some(node) = self.classes.get(derived) {
            if node.bases.contains(&base.to_string()) {
                return true;
            }

            for parent in &node.bases {
                if self.is_base_of(base, parent) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for ClassHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
