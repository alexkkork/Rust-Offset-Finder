// Tue Jan 13 2026 - Alex

use crate::structure::{Field, Offset};

pub struct Member {
    field: Field,
    parent: String,
}

impl Member {
    pub fn new(field: Field, parent: String) -> Self {
        Self { field, parent }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn parent(&self) -> &str {
        &self.parent
    }

    pub fn offset(&self) -> Offset {
        self.field.offset()
    }
}
