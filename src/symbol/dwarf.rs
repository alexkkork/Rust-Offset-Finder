// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// DWARF debug info parser (simplified implementation for ARM64/Mach-O)
pub struct DwarfParser {
    reader: Arc<dyn MemoryReader>,
    compilation_units: Vec<CompilationUnit>,
    type_cache: HashMap<u64, DwarfType>,
}

impl DwarfParser {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            compilation_units: Vec::new(),
            type_cache: HashMap::new(),
        }
    }

    /// Parse DWARF debug info from a section
    pub fn parse(&mut self, debug_info_addr: Address, debug_info_size: usize) -> Result<(), DwarfError> {
        let mut offset = 0;

        while offset < debug_info_size {
            let cu = self.parse_compilation_unit(debug_info_addr + offset as u64)?;
            offset += cu.length as usize + 4; // 4 bytes for length field
            self.compilation_units.push(cu);
        }

        Ok(())
    }

    /// Parse a single compilation unit
    fn parse_compilation_unit(&self, addr: Address) -> Result<CompilationUnit, DwarfError> {
        // Read DWARF compilation unit header
        let length = self.reader.read_u32(addr)
            .map_err(|e| DwarfError::ReadError(e.to_string()))?;
        
        let version = self.reader.read_u16(addr + 4)
            .map_err(|e| DwarfError::ReadError(e.to_string()))?;

        // DWARF 4 vs DWARF 5 have different header layouts
        let (abbrev_offset, addr_size, header_size) = if version <= 4 {
            let abbrev = self.reader.read_u32(addr + 6)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            let size = self.reader.read_u8(addr + 10)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            (abbrev as u64, size, 11)
        } else {
            // DWARF 5
            let unit_type = self.reader.read_u8(addr + 6)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            let size = self.reader.read_u8(addr + 7)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            let abbrev = self.reader.read_u32(addr + 8)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            (abbrev as u64, size, 12)
        };

        Ok(CompilationUnit {
            offset: addr.as_u64(),
            length,
            version,
            abbrev_offset,
            address_size: addr_size,
            entries: Vec::new(),
            source_file: None,
            producer: None,
            language: None,
            low_pc: None,
            high_pc: None,
        })
    }

    /// Parse DIE (Debug Information Entry)
    fn parse_die(&self, addr: Address, abbrev_table: &AbbrevTable) -> Result<DebugEntry, DwarfError> {
        let abbrev_code = self.read_uleb128(addr)?;
        
        if abbrev_code == 0 {
            return Ok(DebugEntry::null());
        }

        let abbrev = abbrev_table.get(abbrev_code as usize)
            .ok_or(DwarfError::InvalidAbbreviation(abbrev_code as usize))?;

        let mut entry = DebugEntry {
            tag: abbrev.tag,
            attributes: Vec::new(),
            children: Vec::new(),
            has_children: abbrev.has_children,
        };

        // Parse attributes (simplified)
        for (attr_name, attr_form) in &abbrev.attributes {
            let value = self.parse_attribute_value(*attr_form)?;
            entry.attributes.push((*attr_name, value));
        }

        Ok(entry)
    }

    fn parse_attribute_value(&self, _form: AttributeForm) -> Result<AttributeValue, DwarfError> {
        // Simplified - would need full form parsing
        Ok(AttributeValue::Unknown)
    }

    fn read_uleb128(&self, addr: Address) -> Result<u64, DwarfError> {
        let mut result: u64 = 0;
        let mut shift = 0;
        let mut byte_offset = 0u64;

        loop {
            let byte = self.reader.read_u8(addr + byte_offset)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            byte_offset += 1;
            
            if shift >= 64 {
                return Err(DwarfError::InvalidEncoding("ULEB128 too large".to_string()));
            }
        }

        Ok(result)
    }

    fn read_sleb128(&self, addr: Address) -> Result<i64, DwarfError> {
        let mut result: i64 = 0;
        let mut shift = 0;
        let mut byte_offset = 0u64;
        let mut byte: u8;

        loop {
            byte = self.reader.read_u8(addr + byte_offset)
                .map_err(|e| DwarfError::ReadError(e.to_string()))?;
            result |= ((byte & 0x7F) as i64) << shift;
            shift += 7;
            byte_offset += 1;

            if byte & 0x80 == 0 {
                break;
            }

            if shift >= 64 {
                return Err(DwarfError::InvalidEncoding("SLEB128 too large".to_string()));
            }
        }

        // Sign extend
        if shift < 64 && (byte & 0x40) != 0 {
            result |= !0i64 << shift;
        }

        Ok(result)
    }

    /// Get all functions from DWARF info
    pub fn get_functions(&self) -> Vec<DwarfFunction> {
        let mut functions = Vec::new();

        for cu in &self.compilation_units {
            for entry in &cu.entries {
                if entry.tag == DwarfTag::Subprogram {
                    if let Some(func) = self.entry_to_function(entry) {
                        functions.push(func);
                    }
                }
            }
        }

        functions
    }

    fn entry_to_function(&self, entry: &DebugEntry) -> Option<DwarfFunction> {
        let mut func = DwarfFunction::new();

        for (attr, value) in &entry.attributes {
            match attr {
                AttributeName::Name => {
                    if let AttributeValue::String(s) = value {
                        func.name = Some(s.clone());
                    }
                }
                AttributeName::LowPc => {
                    if let AttributeValue::Address(a) = value {
                        func.low_pc = Some(*a);
                    }
                }
                AttributeName::HighPc => {
                    if let AttributeValue::Address(a) = value {
                        func.high_pc = Some(*a);
                    } else if let AttributeValue::Unsigned(size) = value {
                        if let Some(low) = func.low_pc {
                            func.high_pc = Some(low + *size);
                        }
                    }
                }
                AttributeName::DeclFile => {
                    if let AttributeValue::Unsigned(idx) = value {
                        func.decl_file = Some(*idx as usize);
                    }
                }
                AttributeName::DeclLine => {
                    if let AttributeValue::Unsigned(line) = value {
                        func.decl_line = Some(*line as usize);
                    }
                }
                _ => {}
            }
        }

        if func.name.is_some() || func.low_pc.is_some() {
            Some(func)
        } else {
            None
        }
    }

    /// Get all types from DWARF info
    pub fn get_types(&self) -> Vec<DwarfType> {
        let mut types = Vec::new();

        for cu in &self.compilation_units {
            for entry in &cu.entries {
                if let Some(typ) = self.entry_to_type(entry) {
                    types.push(typ);
                }
            }
        }

        types
    }

    fn entry_to_type(&self, entry: &DebugEntry) -> Option<DwarfType> {
        match entry.tag {
            DwarfTag::BaseType |
            DwarfTag::PointerType |
            DwarfTag::StructureType |
            DwarfTag::ClassType |
            DwarfTag::UnionType |
            DwarfTag::EnumerationType |
            DwarfTag::ArrayType |
            DwarfTag::TypeDef => {
                Some(DwarfType {
                    tag: entry.tag,
                    name: self.get_string_attribute(entry, AttributeName::Name),
                    byte_size: self.get_unsigned_attribute(entry, AttributeName::ByteSize).map(|v| v as usize),
                    encoding: self.get_unsigned_attribute(entry, AttributeName::Encoding).map(|v| v as u8),
                    members: Vec::new(),
                })
            }
            _ => None
        }
    }

    fn get_string_attribute(&self, entry: &DebugEntry, name: AttributeName) -> Option<String> {
        for (attr, value) in &entry.attributes {
            if *attr == name {
                if let AttributeValue::String(s) = value {
                    return Some(s.clone());
                }
            }
        }
        None
    }

    fn get_unsigned_attribute(&self, entry: &DebugEntry, name: AttributeName) -> Option<u64> {
        for (attr, value) in &entry.attributes {
            if *attr == name {
                if let AttributeValue::Unsigned(v) = value {
                    return Some(*v);
                }
            }
        }
        None
    }

    /// Get variables from DWARF info
    pub fn get_variables(&self) -> Vec<DwarfVariable> {
        let mut variables = Vec::new();

        for cu in &self.compilation_units {
            for entry in &cu.entries {
                if entry.tag == DwarfTag::Variable {
                    if let Some(var) = self.entry_to_variable(entry) {
                        variables.push(var);
                    }
                }
            }
        }

        variables
    }

    fn entry_to_variable(&self, entry: &DebugEntry) -> Option<DwarfVariable> {
        let name = self.get_string_attribute(entry, AttributeName::Name)?;
        
        Some(DwarfVariable {
            name,
            type_offset: self.get_unsigned_attribute(entry, AttributeName::Type),
            location: None,
            is_external: false,
        })
    }

    pub fn compilation_units(&self) -> &[CompilationUnit] {
        &self.compilation_units
    }
}

/// DWARF compilation unit
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub offset: u64,
    pub length: u32,
    pub version: u16,
    pub abbrev_offset: u64,
    pub address_size: u8,
    pub entries: Vec<DebugEntry>,
    pub source_file: Option<String>,
    pub producer: Option<String>,
    pub language: Option<u16>,
    pub low_pc: Option<u64>,
    pub high_pc: Option<u64>,
}

/// Debug Information Entry
#[derive(Debug, Clone)]
pub struct DebugEntry {
    pub tag: DwarfTag,
    pub attributes: Vec<(AttributeName, AttributeValue)>,
    pub children: Vec<DebugEntry>,
    pub has_children: bool,
}

impl DebugEntry {
    pub fn null() -> Self {
        Self {
            tag: DwarfTag::Unknown(0),
            attributes: Vec::new(),
            children: Vec::new(),
            has_children: false,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self.tag, DwarfTag::Unknown(0))
    }
}

/// DWARF tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwarfTag {
    CompileUnit,
    Subprogram,
    Variable,
    FormalParameter,
    LexicalBlock,
    BaseType,
    PointerType,
    ReferenceType,
    StructureType,
    ClassType,
    UnionType,
    EnumerationType,
    ArrayType,
    TypeDef,
    Member,
    InlinedSubroutine,
    Namespace,
    Unknown(u16),
}

impl DwarfTag {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x11 => DwarfTag::CompileUnit,
            0x2E => DwarfTag::Subprogram,
            0x34 => DwarfTag::Variable,
            0x05 => DwarfTag::FormalParameter,
            0x0B => DwarfTag::LexicalBlock,
            0x24 => DwarfTag::BaseType,
            0x0F => DwarfTag::PointerType,
            0x10 => DwarfTag::ReferenceType,
            0x13 => DwarfTag::StructureType,
            0x02 => DwarfTag::ClassType,
            0x17 => DwarfTag::UnionType,
            0x04 => DwarfTag::EnumerationType,
            0x01 => DwarfTag::ArrayType,
            0x16 => DwarfTag::TypeDef,
            0x0D => DwarfTag::Member,
            0x1D => DwarfTag::InlinedSubroutine,
            0x39 => DwarfTag::Namespace,
            _ => DwarfTag::Unknown(value),
        }
    }
}

/// Attribute names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeName {
    Name,
    LowPc,
    HighPc,
    DeclFile,
    DeclLine,
    ByteSize,
    Encoding,
    Type,
    Location,
    External,
    Unknown(u16),
}

impl AttributeName {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x03 => AttributeName::Name,
            0x11 => AttributeName::LowPc,
            0x12 => AttributeName::HighPc,
            0x3A => AttributeName::DeclFile,
            0x3B => AttributeName::DeclLine,
            0x0B => AttributeName::ByteSize,
            0x3E => AttributeName::Encoding,
            0x49 => AttributeName::Type,
            0x02 => AttributeName::Location,
            0x3F => AttributeName::External,
            _ => AttributeName::Unknown(value),
        }
    }
}

/// Attribute form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeForm {
    Addr,
    Data1,
    Data2,
    Data4,
    Data8,
    Strp,
    String,
    Ref4,
    SecOffset,
    Exprloc,
    Flag,
    FlagPresent,
    Unknown(u8),
}

impl AttributeForm {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => AttributeForm::Addr,
            0x0B => AttributeForm::Data1,
            0x05 => AttributeForm::Data2,
            0x06 => AttributeForm::Data4,
            0x07 => AttributeForm::Data8,
            0x0E => AttributeForm::Strp,
            0x08 => AttributeForm::String,
            0x13 => AttributeForm::Ref4,
            0x17 => AttributeForm::SecOffset,
            0x18 => AttributeForm::Exprloc,
            0x0C => AttributeForm::Flag,
            0x19 => AttributeForm::FlagPresent,
            _ => AttributeForm::Unknown(value),
        }
    }
}

/// Attribute value
#[derive(Debug, Clone)]
pub enum AttributeValue {
    Address(u64),
    Unsigned(u64),
    Signed(i64),
    String(String),
    Reference(u64),
    Block(Vec<u8>),
    Flag(bool),
    Unknown,
}

/// Abbreviation table entry
#[derive(Debug, Clone)]
pub struct Abbreviation {
    pub code: usize,
    pub tag: DwarfTag,
    pub has_children: bool,
    pub attributes: Vec<(AttributeName, AttributeForm)>,
}

/// Abbreviation table
pub struct AbbrevTable {
    entries: HashMap<usize, Abbreviation>,
}

impl AbbrevTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add(&mut self, abbrev: Abbreviation) {
        self.entries.insert(abbrev.code, abbrev);
    }

    pub fn get(&self, code: usize) -> Option<&Abbreviation> {
        self.entries.get(&code)
    }
}

impl Default for AbbrevTable {
    fn default() -> Self {
        Self::new()
    }
}

/// DWARF function info
#[derive(Debug, Clone)]
pub struct DwarfFunction {
    pub name: Option<String>,
    pub low_pc: Option<u64>,
    pub high_pc: Option<u64>,
    pub decl_file: Option<usize>,
    pub decl_line: Option<usize>,
    pub parameters: Vec<DwarfParameter>,
    pub return_type: Option<u64>,
    pub is_inline: bool,
    pub is_declaration: bool,
}

impl DwarfFunction {
    pub fn new() -> Self {
        Self {
            name: None,
            low_pc: None,
            high_pc: None,
            decl_file: None,
            decl_line: None,
            parameters: Vec::new(),
            return_type: None,
            is_inline: false,
            is_declaration: false,
        }
    }

    pub fn size(&self) -> Option<u64> {
        match (self.low_pc, self.high_pc) {
            (Some(low), Some(high)) => Some(high - low),
            _ => None,
        }
    }
}

impl Default for DwarfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DwarfFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("<unnamed>");
        write!(f, "{}", name)?;
        if let Some(low) = self.low_pc {
            write!(f, " @ 0x{:x}", low)?;
        }
        if let Some(size) = self.size() {
            write!(f, " ({} bytes)", size)?;
        }
        Ok(())
    }
}

/// DWARF parameter info
#[derive(Debug, Clone)]
pub struct DwarfParameter {
    pub name: Option<String>,
    pub type_offset: Option<u64>,
    pub location: Option<Vec<u8>>,
}

/// DWARF type info
#[derive(Debug, Clone)]
pub struct DwarfType {
    pub tag: DwarfTag,
    pub name: Option<String>,
    pub byte_size: Option<usize>,
    pub encoding: Option<u8>,
    pub members: Vec<DwarfMember>,
}

impl DwarfType {
    pub fn is_base_type(&self) -> bool {
        self.tag == DwarfTag::BaseType
    }

    pub fn is_pointer(&self) -> bool {
        self.tag == DwarfTag::PointerType
    }

    pub fn is_composite(&self) -> bool {
        matches!(self.tag, DwarfTag::StructureType | DwarfTag::ClassType | DwarfTag::UnionType)
    }
}

impl fmt::Display for DwarfType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("<unnamed>");
        write!(f, "{:?} {}", self.tag, name)?;
        if let Some(size) = self.byte_size {
            write!(f, " ({} bytes)", size)?;
        }
        Ok(())
    }
}

/// DWARF member info
#[derive(Debug, Clone)]
pub struct DwarfMember {
    pub name: Option<String>,
    pub type_offset: Option<u64>,
    pub data_member_location: Option<usize>,
}

/// DWARF variable info
#[derive(Debug, Clone)]
pub struct DwarfVariable {
    pub name: String,
    pub type_offset: Option<u64>,
    pub location: Option<Vec<u8>>,
    pub is_external: bool,
}

impl fmt::Display for DwarfVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.is_external {
            write!(f, " [external]")?;
        }
        Ok(())
    }
}

/// DWARF error types
#[derive(Debug, Clone)]
pub enum DwarfError {
    ReadError(String),
    InvalidFormat(String),
    InvalidEncoding(String),
    InvalidAbbreviation(usize),
    UnsupportedVersion(u16),
}

impl fmt::Display for DwarfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DwarfError::ReadError(msg) => write!(f, "Read error: {}", msg),
            DwarfError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DwarfError::InvalidEncoding(msg) => write!(f, "Invalid encoding: {}", msg),
            DwarfError::InvalidAbbreviation(code) => write!(f, "Invalid abbreviation: {}", code),
            DwarfError::UnsupportedVersion(v) => write!(f, "Unsupported DWARF version: {}", v),
        }
    }
}

impl std::error::Error for DwarfError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwarf_tag_from_u16() {
        assert_eq!(DwarfTag::from_u16(0x11), DwarfTag::CompileUnit);
        assert_eq!(DwarfTag::from_u16(0x2E), DwarfTag::Subprogram);
        assert!(matches!(DwarfTag::from_u16(0xFF), DwarfTag::Unknown(_)));
    }

    #[test]
    fn test_dwarf_function_size() {
        let mut func = DwarfFunction::new();
        func.low_pc = Some(0x1000);
        func.high_pc = Some(0x1100);
        
        assert_eq!(func.size(), Some(0x100));
    }
}
