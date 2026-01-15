// Wed Jan 15 2026 - Alex

use super::{DataType, PrimitiveType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeType {
    pub name: String,
    pub fields: Vec<Field>,
    pub is_union: bool,
    pub alignment: usize,
    pub packed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub offset: usize,
    pub bit_field: Option<BitField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitField {
    pub bit_offset: u8,
    pub bit_width: u8,
}

impl CompositeType {
    pub fn new_struct(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            is_union: false,
            alignment: 1,
            packed: false,
        }
    }

    pub fn new_union(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
            is_union: true,
            alignment: 1,
            packed: false,
        }
    }

    pub fn packed(mut self) -> Self {
        self.packed = true;
        self
    }

    pub fn add_field(&mut self, name: &str, data_type: DataType) {
        let offset = if self.is_union {
            0
        } else {
            self.calculate_next_offset(&data_type)
        };

        let field_alignment = data_type.alignment();
        if field_alignment > self.alignment {
            self.alignment = field_alignment;
        }

        self.fields.push(Field {
            name: name.to_string(),
            data_type,
            offset,
            bit_field: None,
        });
    }

    pub fn add_field_at_offset(&mut self, name: &str, data_type: DataType, offset: usize) {
        let field_alignment = data_type.alignment();
        if field_alignment > self.alignment {
            self.alignment = field_alignment;
        }

        self.fields.push(Field {
            name: name.to_string(),
            data_type,
            offset,
            bit_field: None,
        });
    }

    pub fn add_bit_field(&mut self, name: &str, base_type: PrimitiveType, bit_width: u8) {
        let current_size = self.size();
        let bit_offset = (current_size * 8) as u8 % (base_type.size() as u8 * 8);

        self.fields.push(Field {
            name: name.to_string(),
            data_type: DataType::Primitive(base_type),
            offset: current_size,
            bit_field: Some(BitField { bit_offset, bit_width }),
        });
    }

    fn calculate_next_offset(&self, data_type: &DataType) -> usize {
        if self.packed {
            return self.size();
        }

        let current_size = self.size();
        let alignment = data_type.alignment();
        let padding = (alignment - (current_size % alignment)) % alignment;
        current_size + padding
    }

    pub fn size(&self) -> usize {
        if self.is_union {
            self.fields.iter()
                .map(|f| f.data_type.size())
                .max()
                .unwrap_or(0)
        } else {
            self.fields.iter()
                .map(|f| f.offset + f.data_type.size())
                .max()
                .unwrap_or(0)
        }
    }

    pub fn padded_size(&self) -> usize {
        let size = self.size();
        let padding = (self.alignment - (size % self.alignment)) % self.alignment;
        size + padding
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn get_field_at_offset(&self, offset: usize) -> Option<&Field> {
        self.fields.iter().find(|f| {
            offset >= f.offset && offset < f.offset + f.data_type.size()
        })
    }

    pub fn field_offsets(&self) -> HashMap<String, usize> {
        self.fields.iter()
            .map(|f| (f.name.clone(), f.offset))
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() {
            return Ok(());
        }

        if !self.is_union {
            for i in 0..self.fields.len() - 1 {
                let field = &self.fields[i];
                let next_field = &self.fields[i + 1];

                if field.offset + field.data_type.size() > next_field.offset {
                    return Err(format!(
                        "Fields '{}' and '{}' overlap",
                        field.name, next_field.name
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Field {
    pub fn new(name: &str, data_type: DataType, offset: usize) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            offset,
            bit_field: None,
        }
    }

    pub fn end_offset(&self) -> usize {
        self.offset + self.data_type.size()
    }

    pub fn is_bit_field(&self) -> bool {
        self.bit_field.is_some()
    }
}

pub struct CompositeBuilder {
    composite: CompositeType,
}

impl CompositeBuilder {
    pub fn new_struct(name: &str) -> Self {
        Self {
            composite: CompositeType::new_struct(name),
        }
    }

    pub fn new_union(name: &str) -> Self {
        Self {
            composite: CompositeType::new_union(name),
        }
    }

    pub fn packed(mut self) -> Self {
        self.composite.packed = true;
        self
    }

    pub fn field(mut self, name: &str, data_type: DataType) -> Self {
        self.composite.add_field(name, data_type);
        self
    }

    pub fn field_at(mut self, name: &str, data_type: DataType, offset: usize) -> Self {
        self.composite.add_field_at_offset(name, data_type, offset);
        self
    }

    pub fn build(self) -> CompositeType {
        self.composite
    }
}
