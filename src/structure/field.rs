// Tue Jan 13 2026 - Alex

use crate::structure::{Offset, TypeInfo, Size, Alignment};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    offset: Offset,
    type_info: TypeInfo,
    size: Size,
    alignment: Alignment,
}

impl Field {
    pub fn new(name: String, offset: Offset, type_info: TypeInfo) -> Self {
        let size = Size::new(type_info.size());
        let alignment = Alignment::new(type_info.alignment());
        Self {
            name,
            offset,
            type_info,
            size,
            alignment,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn type_info(&self) -> &TypeInfo {
        &self.type_info
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}: {}", self.name, self.offset, self.type_info)
    }
}
