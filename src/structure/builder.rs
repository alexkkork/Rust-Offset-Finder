// Tue Jan 13 2026 - Alex

use crate::structure::{StructureLayout, Field, Offset, TypeInfo};

pub struct StructureBuilder {
    layout: StructureLayout,
}

impl StructureBuilder {
    pub fn new(name: String) -> Self {
        Self {
            layout: StructureLayout::new(name),
        }
    }

    pub fn add_field(mut self, name: String, offset: Offset, type_info: TypeInfo) -> Self {
        let field = Field::new(name, offset, type_info);
        self.layout.add_field(field);
        self
    }

    pub fn build(self) -> StructureLayout {
        self.layout
    }
}
