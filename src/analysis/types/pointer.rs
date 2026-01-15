// Wed Jan 15 2026 - Alex

use super::DataType;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerType {
    pub pointee: DataType,
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

impl PointerType {
    pub fn new(pointee: DataType) -> Self {
        Self {
            pointee,
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }

    pub fn const_ptr(mut self) -> Self {
        self.is_const = true;
        self
    }

    pub fn volatile_ptr(mut self) -> Self {
        self.is_volatile = true;
        self
    }

    pub fn restrict_ptr(mut self) -> Self {
        self.is_restrict = true;
        self
    }

    pub fn void_ptr() -> Self {
        Self::new(DataType::Void)
    }

    pub fn to_void() -> DataType {
        DataType::Pointer(Box::new(Self::void_ptr()))
    }

    pub fn to_u8() -> DataType {
        DataType::Pointer(Box::new(Self::new(DataType::u8())))
    }

    pub fn to_i8() -> DataType {
        DataType::Pointer(Box::new(Self::new(DataType::i8())))
    }

    pub fn to_u64() -> DataType {
        DataType::Pointer(Box::new(Self::new(DataType::u64())))
    }

    pub fn size() -> usize {
        8
    }

    pub fn alignment() -> usize {
        8
    }

    pub fn is_void_pointer(&self) -> bool {
        matches!(self.pointee, DataType::Void)
    }

    pub fn is_char_pointer(&self) -> bool {
        matches!(self.pointee, DataType::Primitive(p) if p == super::PrimitiveType::I8 || p == super::PrimitiveType::Char)
    }

    pub fn dereference_level(&self) -> usize {
        let mut level = 1;
        let mut current = &self.pointee;

        while let DataType::Pointer(inner) = current {
            level += 1;
            current = &inner.pointee;
        }

        level
    }

    pub fn ultimate_pointee(&self) -> &DataType {
        let mut current = &self.pointee;

        while let DataType::Pointer(inner) = current {
            current = &inner.pointee;
        }

        current
    }
}

impl fmt::Display for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut qualifiers = Vec::new();
        if self.is_const {
            qualifiers.push("const");
        }
        if self.is_volatile {
            qualifiers.push("volatile");
        }
        if self.is_restrict {
            qualifiers.push("restrict");
        }

        if qualifiers.is_empty() {
            write!(f, "{}*", self.pointee)
        } else {
            write!(f, "{} {}*", qualifiers.join(" "), self.pointee)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceType {
    pub referent: DataType,
    pub is_lvalue: bool,
}

impl ReferenceType {
    pub fn lvalue(referent: DataType) -> Self {
        Self {
            referent,
            is_lvalue: true,
        }
    }

    pub fn rvalue(referent: DataType) -> Self {
        Self {
            referent,
            is_lvalue: false,
        }
    }

    pub fn size() -> usize {
        8
    }

    pub fn alignment() -> usize {
        8
    }
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_lvalue {
            write!(f, "{}&", self.referent)
        } else {
            write!(f, "{}&&", self.referent)
        }
    }
}

pub struct PointerAnalyzer;

impl PointerAnalyzer {
    pub fn is_likely_pointer(value: u64, base: u64, size: u64) -> bool {
        if value == 0 {
            return false;
        }

        if value < 0x1000 {
            return false;
        }

        if value >= base && value < base + size {
            return true;
        }

        if value & 0x7 != 0 {
            return false;
        }

        let top_bits = value >> 48;
        top_bits == 0 || top_bits == 0xFFFF
    }

    pub fn likely_pointer_target(value: u64, text_base: u64, text_size: u64, data_base: u64, data_size: u64) -> PointerTarget {
        if value == 0 {
            return PointerTarget::Null;
        }

        if value >= text_base && value < text_base + text_size {
            return PointerTarget::Code;
        }

        if value >= data_base && value < data_base + data_size {
            return PointerTarget::Data;
        }

        if Self::is_likely_pointer(value, 0, u64::MAX) {
            return PointerTarget::Heap;
        }

        PointerTarget::Invalid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerTarget {
    Null,
    Code,
    Data,
    Stack,
    Heap,
    Invalid,
}
