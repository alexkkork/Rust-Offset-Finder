// Tue Jan 13 2026 - Alex

use crate::structure::{Field, Offset, Size, Alignment};
use std::collections::HashMap;

#[derive(Clone)]
pub struct StructureLayout {
    name: String,
    fields: Vec<Field>,
    field_map: HashMap<String, usize>,
    size: Size,
    alignment: Alignment,
}

impl StructureLayout {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            field_map: HashMap::new(),
            size: Size::zero(),
            alignment: Alignment::default(),
        }
    }

    pub fn add_field(&mut self, field: Field) {
        let index = self.fields.len();
        self.field_map.insert(field.name().to_string(), index);
        self.fields.push(field);
        self.recalculate_size();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.field_map.get(name).map(|&idx| &self.fields[idx])
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    fn recalculate_size(&mut self) {
        if self.fields.is_empty() {
            self.size = Size::zero();
            self.alignment = Alignment::default();
            return;
        }

        let max_align = self.fields.iter().map(|f| f.alignment().as_usize()).max().unwrap_or(8);
        self.alignment = Alignment::new(max_align);

        let mut current_offset = 0u64;
        for field in &self.fields {
            let align = field.alignment().as_usize() as u64;
            current_offset = (current_offset + align - 1) & !(align - 1);
            current_offset += field.size().as_u64();
        }
        current_offset = (current_offset + max_align as u64 - 1) & !(max_align as u64 - 1);
        self.size = Size::new(current_offset as usize);
    }
}
