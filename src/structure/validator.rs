// Tue Jan 13 2026 - Alex

use crate::structure::{StructureLayout, StructureError};

pub struct StructureValidator;

impl StructureValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, layout: &StructureLayout) -> Result<(), StructureError> {
        for field in layout.fields() {
            if field.offset().as_u64() % field.alignment().as_usize() as u64 != 0 {
                return Err(StructureError::ValidationFailed(format!("Field {} not aligned", field.name())));
            }
        }
        Ok(())
    }
}

impl Default for StructureValidator {
    fn default() -> Self {
        Self::new()
    }
}
