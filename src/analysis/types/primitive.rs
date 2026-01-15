// Wed Jan 15 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Char,
    USize,
    ISize,
}

impl PrimitiveType {
    pub fn size(&self) -> usize {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::I8 | PrimitiveType::Char => 1,
            PrimitiveType::U16 | PrimitiveType::I16 => 2,
            PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32 => 4,
            PrimitiveType::U64 | PrimitiveType::I64 | PrimitiveType::F64 |
            PrimitiveType::USize | PrimitiveType::ISize => 8,
            PrimitiveType::U128 | PrimitiveType::I128 => 16,
        }
    }

    pub fn alignment(&self) -> usize {
        self.size()
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, 
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 |
            PrimitiveType::I64 | PrimitiveType::I128 | PrimitiveType::ISize
        )
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self,
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::U16 |
            PrimitiveType::U32 | PrimitiveType::U64 | PrimitiveType::U128 |
            PrimitiveType::USize | PrimitiveType::Char
        )
    }

    pub fn is_integer(&self) -> bool {
        !matches!(self, PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::Bool)
    }

    pub fn is_floating(&self) -> bool {
        matches!(self, PrimitiveType::F32 | PrimitiveType::F64)
    }

    pub fn is_numeric(&self) -> bool {
        !matches!(self, PrimitiveType::Bool | PrimitiveType::Char)
    }

    pub fn min_value(&self) -> i128 {
        match self {
            PrimitiveType::Bool => 0,
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 |
            PrimitiveType::U64 | PrimitiveType::U128 | PrimitiveType::USize |
            PrimitiveType::Char => 0,
            PrimitiveType::I8 => i8::MIN as i128,
            PrimitiveType::I16 => i16::MIN as i128,
            PrimitiveType::I32 => i32::MIN as i128,
            PrimitiveType::I64 | PrimitiveType::ISize => i64::MIN as i128,
            PrimitiveType::I128 => i128::MIN,
            PrimitiveType::F32 | PrimitiveType::F64 => 0,
        }
    }

    pub fn max_value(&self) -> u128 {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::U8 | PrimitiveType::Char => u8::MAX as u128,
            PrimitiveType::U16 => u16::MAX as u128,
            PrimitiveType::U32 => u32::MAX as u128,
            PrimitiveType::U64 | PrimitiveType::USize => u64::MAX as u128,
            PrimitiveType::U128 => u128::MAX,
            PrimitiveType::I8 => i8::MAX as u128,
            PrimitiveType::I16 => i16::MAX as u128,
            PrimitiveType::I32 => i32::MAX as u128,
            PrimitiveType::I64 | PrimitiveType::ISize => i64::MAX as u128,
            PrimitiveType::I128 => i128::MAX as u128,
            PrimitiveType::F32 | PrimitiveType::F64 => 0,
        }
    }

    pub fn from_c_type(name: &str) -> Option<Self> {
        match name {
            "bool" | "_Bool" => Some(PrimitiveType::Bool),
            "char" | "signed char" => Some(PrimitiveType::I8),
            "unsigned char" => Some(PrimitiveType::U8),
            "short" | "signed short" | "short int" => Some(PrimitiveType::I16),
            "unsigned short" | "unsigned short int" => Some(PrimitiveType::U16),
            "int" | "signed" | "signed int" => Some(PrimitiveType::I32),
            "unsigned" | "unsigned int" => Some(PrimitiveType::U32),
            "long" | "signed long" | "long int" => Some(PrimitiveType::I64),
            "unsigned long" | "unsigned long int" => Some(PrimitiveType::U64),
            "long long" | "signed long long" => Some(PrimitiveType::I64),
            "unsigned long long" => Some(PrimitiveType::U64),
            "float" => Some(PrimitiveType::F32),
            "double" => Some(PrimitiveType::F64),
            "size_t" => Some(PrimitiveType::USize),
            "ssize_t" | "ptrdiff_t" => Some(PrimitiveType::ISize),
            "int8_t" => Some(PrimitiveType::I8),
            "int16_t" => Some(PrimitiveType::I16),
            "int32_t" => Some(PrimitiveType::I32),
            "int64_t" => Some(PrimitiveType::I64),
            "uint8_t" => Some(PrimitiveType::U8),
            "uint16_t" => Some(PrimitiveType::U16),
            "uint32_t" => Some(PrimitiveType::U32),
            "uint64_t" => Some(PrimitiveType::U64),
            _ => None,
        }
    }

    pub fn to_c_type(&self) -> &'static str {
        match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "uint8_t",
            PrimitiveType::U16 => "uint16_t",
            PrimitiveType::U32 => "uint32_t",
            PrimitiveType::U64 => "uint64_t",
            PrimitiveType::U128 => "__uint128_t",
            PrimitiveType::I8 => "int8_t",
            PrimitiveType::I16 => "int16_t",
            PrimitiveType::I32 => "int32_t",
            PrimitiveType::I64 => "int64_t",
            PrimitiveType::I128 => "__int128_t",
            PrimitiveType::F32 => "float",
            PrimitiveType::F64 => "double",
            PrimitiveType::Char => "char",
            PrimitiveType::USize => "size_t",
            PrimitiveType::ISize => "ssize_t",
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::U128 => write!(f, "u128"),
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::I128 => write!(f, "i128"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::USize => write!(f, "usize"),
            PrimitiveType::ISize => write!(f, "isize"),
        }
    }
}
