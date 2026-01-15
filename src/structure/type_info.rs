// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    Primitive(PrimitiveType),
    Pointer(Box<TypeInfo>),
    Array(Box<TypeInfo>, usize),
    Struct(Vec<(usize, TypeInfo)>),
    Union(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
    Bool,
    Usize,
    Isize,
}

impl TypeInfo {
    pub fn size(&self) -> usize {
        match self {
            Self::Primitive(ty) => ty.size(),
            Self::Pointer(_) => 8,
            Self::Array(elem, count) => elem.size() * count,
            Self::Struct(fields) => fields.iter().map(|(_, t)| t.size()).sum(),
            Self::Union(_) => 0,
            Self::Unknown => 0,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            Self::Primitive(ty) => ty.alignment(),
            Self::Pointer(_) => 8,
            Self::Array(elem, _) => elem.alignment(),
            Self::Struct(fields) => fields.iter().map(|(_, t)| t.alignment()).max().unwrap_or(1),
            Self::Union(_) => 8,
            Self::Unknown => 1,
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer(_))
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_, _))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl PrimitiveType {
    pub fn size(self) -> usize {
        match self {
            Self::U8 | Self::I8 | Self::Bool => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 | Self::Ptr | Self::Usize | Self::Isize => 8,
        }
    }

    pub fn alignment(self) -> usize {
        match self {
            Self::U8 | Self::I8 | Self::Bool => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 | Self::Ptr | Self::Usize | Self::Isize => 8,
        }
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isize)
    }

    pub fn is_unsigned(self) -> bool {
        matches!(self, Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usize | Self::Bool)
    }

    pub fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    pub fn is_integer(self) -> bool {
        !self.is_float() && self != Self::Bool
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(ty) => write!(f, "{:?}", ty),
            Self::Pointer(inner) => write!(f, "*{}", inner),
            Self::Array(elem, count) => write!(f, "[{}; {}]", elem, count),
            Self::Struct(fields) => {
                write!(f, "struct {{ ")?;
                for (i, (offset, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "@{}: {}", offset, ty)?;
                }
                write!(f, " }}")
            }
            Self::Union(name) => write!(f, "union {}", name),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
