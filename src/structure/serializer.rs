// Tue Jan 13 2026 - Alex

use crate::structure::StructureLayout;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SerializableLayout {
    name: String,
    fields: Vec<SerializableField>,
    size: usize,
    alignment: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableField {
    name: String,
    offset: u64,
    size: usize,
    alignment: usize,
}

impl From<&StructureLayout> for SerializableLayout {
    fn from(layout: &StructureLayout) -> Self {
        Self {
            name: layout.name().to_string(),
            fields: layout.fields().iter().map(|f| SerializableField {
                name: f.name().to_string(),
                offset: f.offset().as_u64(),
                size: f.size().as_usize(),
                alignment: f.alignment().as_usize(),
            }).collect(),
            size: layout.size().as_usize(),
            alignment: layout.alignment().as_usize(),
        }
    }
}
