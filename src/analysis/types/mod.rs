// Wed Jan 15 2026 - Alex

pub mod primitive;
pub mod composite;
pub mod pointer;
pub mod inference;

pub use primitive::PrimitiveType;
pub use composite::CompositeType;
pub use pointer::PointerType;
pub use inference::TypeInferenceEngine;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Primitive(PrimitiveType),
    Pointer(Box<PointerType>),
    Array { element: Box<DataType>, count: usize },
    Struct(CompositeType),
    Union(CompositeType),
    Function(FunctionType),
    Void,
    Unknown,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::Primitive(p) => p.size(),
            DataType::Pointer(_) => 8,
            DataType::Array { element, count } => element.size() * count,
            DataType::Struct(c) | DataType::Union(c) => c.size(),
            DataType::Function(_) => 8,
            DataType::Void => 0,
            DataType::Unknown => 0,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            DataType::Primitive(p) => p.alignment(),
            DataType::Pointer(_) => 8,
            DataType::Array { element, .. } => element.alignment(),
            DataType::Struct(c) | DataType::Union(c) => c.alignment(),
            DataType::Function(_) => 8,
            DataType::Void => 1,
            DataType::Unknown => 1,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, DataType::Primitive(p) if p.is_numeric())
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, DataType::Pointer(_))
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, DataType::Struct(_) | DataType::Union(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, DataType::Array { .. })
    }

    pub fn u8() -> Self {
        DataType::Primitive(PrimitiveType::U8)
    }

    pub fn u16() -> Self {
        DataType::Primitive(PrimitiveType::U16)
    }

    pub fn u32() -> Self {
        DataType::Primitive(PrimitiveType::U32)
    }

    pub fn u64() -> Self {
        DataType::Primitive(PrimitiveType::U64)
    }

    pub fn i8() -> Self {
        DataType::Primitive(PrimitiveType::I8)
    }

    pub fn i16() -> Self {
        DataType::Primitive(PrimitiveType::I16)
    }

    pub fn i32() -> Self {
        DataType::Primitive(PrimitiveType::I32)
    }

    pub fn i64() -> Self {
        DataType::Primitive(PrimitiveType::I64)
    }

    pub fn f32() -> Self {
        DataType::Primitive(PrimitiveType::F32)
    }

    pub fn f64() -> Self {
        DataType::Primitive(PrimitiveType::F64)
    }

    pub fn pointer_to(inner: DataType) -> Self {
        DataType::Pointer(Box::new(PointerType::new(inner)))
    }

    pub fn array_of(element: DataType, count: usize) -> Self {
        DataType::Array { element: Box::new(element), count }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Primitive(p) => write!(f, "{}", p),
            DataType::Pointer(p) => write!(f, "{}*", p.pointee),
            DataType::Array { element, count } => write!(f, "{}[{}]", element, count),
            DataType::Struct(c) => write!(f, "struct {}", c.name),
            DataType::Union(c) => write!(f, "union {}", c.name),
            DataType::Function(ft) => write!(f, "{}", ft),
            DataType::Void => write!(f, "void"),
            DataType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub return_type: Box<DataType>,
    pub parameters: Vec<DataType>,
    pub variadic: bool,
}

impl FunctionType {
    pub fn new(return_type: DataType, parameters: Vec<DataType>) -> Self {
        Self {
            return_type: Box::new(return_type),
            parameters,
            variadic: false,
        }
    }

    pub fn variadic(mut self) -> Self {
        self.variadic = true;
        self
    }
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.return_type)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        if self.variadic {
            if !self.parameters.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "...")?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedValue {
    pub data_type: DataType,
    pub value: Vec<u8>,
}

impl TypedValue {
    pub fn new(data_type: DataType, value: Vec<u8>) -> Self {
        Self { data_type, value }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if self.value.len() >= 8 {
            Some(u64::from_le_bytes([
                self.value[0], self.value[1], self.value[2], self.value[3],
                self.value[4], self.value[5], self.value[6], self.value[7],
            ]))
        } else if self.value.len() >= 4 {
            Some(u32::from_le_bytes([
                self.value[0], self.value[1], self.value[2], self.value[3],
            ]) as u64)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_u64().map(|v| v as i64)
    }

    pub fn as_f64(&self) -> Option<f64> {
        if self.value.len() >= 8 {
            Some(f64::from_le_bytes([
                self.value[0], self.value[1], self.value[2], self.value[3],
                self.value[4], self.value[5], self.value[6], self.value[7],
            ]))
        } else {
            None
        }
    }
}
