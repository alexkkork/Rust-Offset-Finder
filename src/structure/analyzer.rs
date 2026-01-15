// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::structure::{StructureLayout, Field, Offset, TypeInfo, StructureError};
use std::sync::Arc;

pub struct StructureAnalyzer {
    layouts: Vec<StructureLayout>,
    reader: Arc<dyn MemoryReader>,
}

impl StructureAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            layouts: Vec::new(),
            reader,
        }
    }

    pub fn add_layout(&mut self, layout: StructureLayout) {
        self.layouts.push(layout);
    }

    pub fn analyze_structure(&mut self, name: String, base: Address, fields: Vec<(String, TypeInfo)>) -> Result<StructureLayout, StructureError> {
        let mut layout = StructureLayout::new(name);
        let mut current_offset = 0u64;
        for (field_name, type_info) in fields {
            let offset = Offset::new(current_offset);
            let field = Field::new(field_name, offset, type_info.clone());
            layout.add_field(field);
            current_offset += type_info.size() as u64;
            current_offset = (current_offset + type_info.alignment() as u64 - 1) & !(type_info.alignment() as u64 - 1);
        }
        Ok(layout)
    }

    pub fn get_layout(&self, name: &str) -> Option<&StructureLayout> {
        self.layouts.iter().find(|l| l.name() == name)
    }

    pub fn layouts(&self) -> &[StructureLayout] {
        &self.layouts
    }
}
