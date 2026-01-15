// Tue Jan 13 2026 - Alex

use crate::structure::StructureLayout;
use crate::memory::Address;

pub struct StructureTraverser {
    layout: StructureLayout,
}

impl StructureTraverser {
    pub fn new(layout: StructureLayout) -> Self {
        Self { layout }
    }

    pub fn get_field_address(&self, base: Address, field_name: &str) -> Option<Address> {
        self.layout.get_field(field_name)
            .map(|f| base + f.offset().as_u64())
    }

    pub fn traverse_fields(&self, base: Address) -> Vec<(String, Address)> {
        self.layout.fields()
            .iter()
            .map(|f| (f.name().to_string(), base + f.offset().as_u64()))
            .collect()
    }
}
