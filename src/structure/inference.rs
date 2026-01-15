// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::structure::TypeInfo;
use crate::structure::type_info::PrimitiveType;
use std::sync::Arc;
use std::collections::HashMap;

pub struct TypeInference {
    reader: Arc<dyn MemoryReader>,
    cache: HashMap<u64, TypeInfo>,
}

impl TypeInference {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            cache: HashMap::new(),
        }
    }

    pub fn infer_type(&self, address: Address) -> Result<TypeInfo, MemoryError> {
        let value = self.reader.read_u64(address)?;
        if value == 0 {
            Ok(TypeInfo::Pointer(Box::new(TypeInfo::Unknown)))
        } else if value < 0x100000000 {
            Ok(TypeInfo::Primitive(PrimitiveType::U32))
        } else {
            Ok(TypeInfo::Pointer(Box::new(TypeInfo::Unknown)))
        }
    }

    pub fn infer_type_cached(&mut self, address: Address) -> Result<TypeInfo, MemoryError> {
        let addr_u64 = address.as_u64();
        if let Some(cached) = self.cache.get(&addr_u64) {
            return Ok(cached.clone());
        }
        let result = self.infer_type(address)?;
        self.cache.insert(addr_u64, result.clone());
        Ok(result)
    }

    pub fn infer_array_type(&self, address: Address, count: usize) -> Result<TypeInfo, MemoryError> {
        if count == 0 {
            return Ok(TypeInfo::Unknown);
        }

        let first_type = self.infer_type(address)?;
        let element_size = self.estimate_size(&first_type);

        for i in 1..count.min(10) {
            let elem_addr = address + (i * element_size) as u64;
            let elem_type = self.infer_type(elem_addr)?;
            if !self.types_compatible(&first_type, &elem_type) {
                return Ok(TypeInfo::Unknown);
            }
        }

        Ok(TypeInfo::Array(Box::new(first_type), count))
    }

    pub fn infer_struct_layout(&self, address: Address, size: usize) -> Result<Vec<(usize, TypeInfo)>, MemoryError> {
        let mut fields = Vec::new();
        let mut offset = 0;

        while offset < size {
            let field_addr = address + offset as u64;
            let field_type = self.infer_type(field_addr)?;
            let field_size = self.estimate_size(&field_type);

            fields.push((offset, field_type));
            offset += field_size;
        }

        Ok(fields)
    }

    pub fn estimate_size(&self, type_info: &TypeInfo) -> usize {
        match type_info {
            TypeInfo::Primitive(prim) => match prim {
                PrimitiveType::U8 | PrimitiveType::I8 | PrimitiveType::Bool => 1,
                PrimitiveType::U16 | PrimitiveType::I16 => 2,
                PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32 => 4,
                PrimitiveType::U64 | PrimitiveType::I64 | PrimitiveType::F64 | PrimitiveType::Ptr | PrimitiveType::Usize | PrimitiveType::Isize => 8,
            },
            TypeInfo::Pointer(_) => 8,
            TypeInfo::Array(elem, count) => self.estimate_size(elem) * count,
            TypeInfo::Struct(fields) => fields.iter().map(|(_, t)| self.estimate_size(t)).sum(),
            TypeInfo::Union(_) => 8,
            TypeInfo::Unknown => 8,
        }
    }

    pub fn types_compatible(&self, a: &TypeInfo, b: &TypeInfo) -> bool {
        match (a, b) {
            (TypeInfo::Primitive(pa), TypeInfo::Primitive(pb)) => pa == pb,
            (TypeInfo::Pointer(_), TypeInfo::Pointer(_)) => true,
            (TypeInfo::Array(ea, ca), TypeInfo::Array(eb, cb)) => {
                ca == cb && self.types_compatible(ea, eb)
            }
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,
            _ => false,
        }
    }

    pub fn is_likely_pointer(&self, address: Address) -> Result<bool, MemoryError> {
        let value = self.reader.read_u64(address)?;
        Ok(value >= 0x100000000 && value < 0x800000000000)
    }

    pub fn is_likely_string_pointer(&self, address: Address) -> Result<bool, MemoryError> {
        if !self.is_likely_pointer(address)? {
            return Ok(false);
        }

        let ptr = self.reader.read_u64(address)?;
        let ptr_addr = Address::new(ptr);

        match self.reader.read_bytes(ptr_addr, 16) {
            Ok(bytes) => {
                let printable_count = bytes.iter().take_while(|&&b| b >= 0x20 && b < 0x7f).count();
                Ok(printable_count >= 4)
            }
            Err(_) => Ok(false),
        }
    }

    pub fn is_likely_vtable(&self, address: Address) -> Result<bool, MemoryError> {
        let mut pointer_count = 0;
        for i in 0..8 {
            let ptr_addr = address + (i * 8) as u64;
            if self.is_likely_pointer(ptr_addr)? {
                pointer_count += 1;
            }
        }
        Ok(pointer_count >= 6)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

pub struct StructureReconstructor {
    inference: TypeInference,
}

impl StructureReconstructor {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            inference: TypeInference::new(reader),
        }
    }

    pub fn reconstruct(&self, address: Address, size: usize) -> Result<Vec<(usize, String, TypeInfo)>, MemoryError> {
        let layout = self.inference.infer_struct_layout(address, size)?;
        let mut fields = Vec::new();

        for (i, (offset, type_info)) in layout.iter().enumerate() {
            let name = format!("field_{}", i);
            fields.push((*offset, name, type_info.clone()));
        }

        Ok(fields)
    }

    pub fn find_vtable(&self, address: Address) -> Result<Option<u64>, MemoryError> {
        let first_ptr = self.inference.reader.read_u64(address)?;
        let first_ptr_addr = Address::new(first_ptr);

        if self.inference.is_likely_vtable(first_ptr_addr)? {
            return Ok(Some(first_ptr));
        }

        Ok(None)
    }

    pub fn inference(&self) -> &TypeInference {
        &self.inference
    }
}
