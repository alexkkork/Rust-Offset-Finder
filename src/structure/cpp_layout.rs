// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::structure::TypeInfo;
use crate::structure::type_info::PrimitiveType;
use crate::structure::vtable::VTableAnalyzer;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Represents a C++ class member
#[derive(Debug, Clone)]
pub struct CppMember {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub type_info: TypeInfo,
    pub access: AccessSpecifier,
    pub is_static: bool,
    pub is_mutable: bool,
    pub bit_field: Option<BitFieldInfo>,
}

impl CppMember {
    pub fn new(name: &str, offset: usize, type_info: TypeInfo) -> Self {
        Self {
            name: name.to_string(),
            offset,
            size: type_info.size(),
            type_info,
            access: AccessSpecifier::Public,
            is_static: false,
            is_mutable: false,
            bit_field: None,
        }
    }

    pub fn with_access(mut self, access: AccessSpecifier) -> Self {
        self.access = access;
        self
    }

    pub fn with_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn with_mutable(mut self) -> Self {
        self.is_mutable = true;
        self
    }

    pub fn with_bit_field(mut self, bits: usize, position: usize) -> Self {
        self.bit_field = Some(BitFieldInfo { bits, position });
        self
    }

    pub fn end_offset(&self) -> usize {
        self.offset + self.size
    }

    pub fn is_pointer(&self) -> bool {
        self.type_info.is_pointer()
    }

    pub fn is_array(&self) -> bool {
        self.type_info.is_array()
    }
}

impl fmt::Display for CppMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let access = match self.access {
            AccessSpecifier::Public => "public",
            AccessSpecifier::Protected => "protected",
            AccessSpecifier::Private => "private",
        };
        write!(f, "{} {} {} @ 0x{:X}", access, self.type_info, self.name, self.offset)?;
        if let Some(ref bf) = self.bit_field {
            write!(f, " : {} bits at {}", bf.bits, bf.position)?;
        }
        Ok(())
    }
}

/// C++ access specifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSpecifier {
    Public,
    Protected,
    Private,
}

/// Information about a bit field
#[derive(Debug, Clone)]
pub struct BitFieldInfo {
    pub bits: usize,
    pub position: usize,
}

/// Represents a C++ class virtual function
#[derive(Debug, Clone)]
pub struct CppVirtualMethod {
    pub name: String,
    pub vtable_index: usize,
    pub address: Address,
    pub is_pure: bool,
    pub is_override: bool,
    pub is_final: bool,
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
}

impl CppVirtualMethod {
    pub fn new(name: &str, vtable_index: usize, address: Address) -> Self {
        Self {
            name: name.to_string(),
            vtable_index,
            address,
            is_pure: false,
            is_override: false,
            is_final: false,
            return_type: None,
            parameters: Vec::new(),
        }
    }

    pub fn with_pure(mut self) -> Self {
        self.is_pure = true;
        self
    }

    pub fn with_override(mut self) -> Self {
        self.is_override = true;
        self
    }

    pub fn with_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    pub fn with_signature(mut self, return_type: &str, params: Vec<&str>) -> Self {
        self.return_type = Some(return_type.to_string());
        self.parameters = params.iter().map(|s| s.to_string()).collect();
        self
    }
}

impl fmt::Display for CppVirtualMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = self.return_type.as_deref().unwrap_or("void");
        let params = self.parameters.join(", ");
        write!(f, "virtual {} {}({})", ret, self.name, params)?;
        if self.is_pure {
            write!(f, " = 0")?;
        }
        if self.is_override {
            write!(f, " override")?;
        }
        if self.is_final {
            write!(f, " final")?;
        }
        Ok(())
    }
}

/// Represents a base class with offset information
#[derive(Debug, Clone)]
pub struct CppBaseClass {
    pub name: String,
    pub offset: usize,
    pub is_virtual: bool,
    pub access: AccessSpecifier,
}

impl CppBaseClass {
    pub fn new(name: &str, offset: usize) -> Self {
        Self {
            name: name.to_string(),
            offset,
            is_virtual: false,
            access: AccessSpecifier::Public,
        }
    }

    pub fn with_virtual(mut self) -> Self {
        self.is_virtual = true;
        self
    }

    pub fn with_access(mut self, access: AccessSpecifier) -> Self {
        self.access = access;
        self
    }
}

impl fmt::Display for CppBaseClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let access = match self.access {
            AccessSpecifier::Public => "public",
            AccessSpecifier::Protected => "protected",
            AccessSpecifier::Private => "private",
        };
        let virtual_kw = if self.is_virtual { "virtual " } else { "" };
        write!(f, "{}{}{} @ 0x{:X}", virtual_kw, access, self.name, self.offset)
    }
}

/// Complete C++ class layout
#[derive(Debug, Clone)]
pub struct CppClassLayout {
    /// Name of the class
    pub name: String,
    /// Total size of the class
    pub size: usize,
    /// Alignment requirement
    pub alignment: usize,
    /// Base classes
    pub base_classes: Vec<CppBaseClass>,
    /// Data members
    pub members: Vec<CppMember>,
    /// Virtual methods
    pub virtual_methods: Vec<CppVirtualMethod>,
    /// VTable pointer offset (usually 0)
    pub vtable_offset: Option<usize>,
    /// VTable address
    pub vtable_address: Option<Address>,
    /// Whether the class is polymorphic
    pub is_polymorphic: bool,
    /// Whether the class is abstract
    pub is_abstract: bool,
    /// RTTI type info address
    pub rtti_address: Option<Address>,
    /// Padding regions
    pub padding: Vec<(usize, usize)>, // (offset, size)
}

impl CppClassLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            size: 0,
            alignment: 8,
            base_classes: Vec::new(),
            members: Vec::new(),
            virtual_methods: Vec::new(),
            vtable_offset: None,
            vtable_address: None,
            is_polymorphic: false,
            is_abstract: false,
            rtti_address: None,
            padding: Vec::new(),
        }
    }

    pub fn add_base(&mut self, base: CppBaseClass) {
        self.base_classes.push(base);
    }

    pub fn add_member(&mut self, member: CppMember) {
        self.members.push(member);
        self.recalculate_size();
    }

    pub fn add_virtual_method(&mut self, method: CppVirtualMethod) {
        if method.is_pure {
            self.is_abstract = true;
        }
        self.virtual_methods.push(method);
        self.is_polymorphic = true;
    }

    pub fn set_vtable(&mut self, offset: usize, address: Address) {
        self.vtable_offset = Some(offset);
        self.vtable_address = Some(address);
        self.is_polymorphic = true;
    }

    fn recalculate_size(&mut self) {
        if self.members.is_empty() && self.base_classes.is_empty() {
            self.size = 0;
            return;
        }

        let mut max_end = 0;
        for member in &self.members {
            max_end = max_end.max(member.end_offset());
        }

        // Align to class alignment
        self.size = (max_end + self.alignment - 1) & !(self.alignment - 1);
    }

    /// Find padding holes in the layout
    pub fn find_padding(&mut self) {
        self.padding.clear();

        let mut sorted_members: Vec<&CppMember> = self.members.iter().collect();
        sorted_members.sort_by_key(|m| m.offset);

        let mut expected_offset = if self.is_polymorphic { 8 } else { 0 };

        // Account for base classes
        for base in &self.base_classes {
            if base.offset == expected_offset {
                // Would need to know base class size here
                // For now, assume no padding between bases
            }
        }

        for member in sorted_members {
            if member.offset > expected_offset {
                self.padding.push((expected_offset, member.offset - expected_offset));
            }
            expected_offset = member.end_offset();
        }

        // Check for tail padding
        if expected_offset < self.size {
            self.padding.push((expected_offset, self.size - expected_offset));
        }
    }

    /// Get member by name
    pub fn get_member(&self, name: &str) -> Option<&CppMember> {
        self.members.iter().find(|m| m.name == name)
    }

    /// Get member by offset
    pub fn get_member_at_offset(&self, offset: usize) -> Option<&CppMember> {
        self.members.iter().find(|m| m.offset == offset)
    }

    /// Get virtual method by index
    pub fn get_virtual_method(&self, index: usize) -> Option<&CppVirtualMethod> {
        self.virtual_methods.iter().find(|m| m.vtable_index == index)
    }

    /// Total padding bytes
    pub fn total_padding(&self) -> usize {
        self.padding.iter().map(|(_, size)| size).sum()
    }

    /// Padding as percentage of size
    pub fn padding_percentage(&self) -> f64 {
        if self.size == 0 {
            return 0.0;
        }
        (self.total_padding() as f64 / self.size as f64) * 100.0
    }

    /// Generate C++ code representation
    pub fn to_cpp(&self) -> String {
        let mut code = String::new();

        code.push_str(&format!("class {} ", self.name));

        if !self.base_classes.is_empty() {
            code.push_str(": ");
            let bases: Vec<String> = self.base_classes.iter().map(|b| {
                let access = match b.access {
                    AccessSpecifier::Public => "public",
                    AccessSpecifier::Protected => "protected",
                    AccessSpecifier::Private => "private",
                };
                let virtual_kw = if b.is_virtual { "virtual " } else { "" };
                format!("{}{} {}", virtual_kw, access, b.name)
            }).collect();
            code.push_str(&bases.join(", "));
        }

        code.push_str(" {\n");

        // Group by access specifier
        let mut current_access = AccessSpecifier::Private;
        
        // Virtual methods
        if !self.virtual_methods.is_empty() {
            code.push_str("public:\n");
            current_access = AccessSpecifier::Public;
            for method in &self.virtual_methods {
                code.push_str(&format!("    {};\n", method));
            }
            code.push('\n');
        }

        // Members grouped by access
        for access in [AccessSpecifier::Public, AccessSpecifier::Protected, AccessSpecifier::Private] {
            let access_members: Vec<&CppMember> = self.members.iter()
                .filter(|m| m.access == access)
                .collect();
            
            if !access_members.is_empty() {
                if current_access != access {
                    let access_str = match access {
                        AccessSpecifier::Public => "public",
                        AccessSpecifier::Protected => "protected",
                        AccessSpecifier::Private => "private",
                    };
                    code.push_str(&format!("{}:\n", access_str));
                    current_access = access;
                }

                for member in access_members {
                    code.push_str(&format!("    {} {}; // offset 0x{:X}\n", 
                        member.type_info, member.name, member.offset));
                }
            }
        }

        code.push_str(&format!("}}; // sizeof = 0x{:X} ({} bytes)\n", self.size, self.size));

        code
    }
}

impl fmt::Display for CppClassLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class {} {{", self.name)?;
        writeln!(f, "  // Size: 0x{:X} ({} bytes)", self.size, self.size)?;
        writeln!(f, "  // Alignment: {}", self.alignment)?;
        
        if self.is_polymorphic {
            writeln!(f, "  // Polymorphic: yes")?;
            if let Some(offset) = self.vtable_offset {
                write!(f, "  // VTable @ offset 0x{:X}", offset)?;
                if let Some(addr) = self.vtable_address {
                    write!(f, " (address: {:016x})", addr.as_u64())?;
                }
                writeln!(f)?;
            }
        }

        if !self.base_classes.is_empty() {
            writeln!(f, "  // Base classes:")?;
            for base in &self.base_classes {
                writeln!(f, "  //   {}", base)?;
            }
        }

        if !self.virtual_methods.is_empty() {
            writeln!(f, "  // Virtual methods:")?;
            for method in &self.virtual_methods {
                writeln!(f, "  //   [{}] {}", method.vtable_index, method.name)?;
            }
        }

        writeln!(f, "  // Members:")?;
        for member in &self.members {
            writeln!(f, "  {}", member)?;
        }

        if !self.padding.is_empty() {
            writeln!(f, "  // Padding ({} bytes total, {:.1}%):", 
                self.total_padding(), self.padding_percentage())?;
            for (offset, size) in &self.padding {
                writeln!(f, "  //   0x{:X} - 0x{:X} ({} bytes)", offset, offset + size, size)?;
            }
        }

        writeln!(f, "}}")
    }
}

/// Reconstructs C++ class layouts from memory
pub struct CppLayoutReconstructor {
    reader: Arc<dyn MemoryReader>,
    vtable_analyzer: VTableAnalyzer,
    layouts: HashMap<String, CppClassLayout>,
}

impl CppLayoutReconstructor {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            vtable_analyzer: VTableAnalyzer::new(reader),
            layouts: HashMap::new(),
        }
    }

    /// Reconstruct a class layout from an instance address
    pub fn reconstruct_from_instance(&mut self, instance_addr: Address, class_name: &str, estimated_size: usize) -> Result<CppClassLayout, MemoryError> {
        let mut layout = CppClassLayout::new(class_name);

        // Read vtable pointer (usually at offset 0)
        let vtable_ptr = self.reader.read_u64(instance_addr)?;
        if vtable_ptr >= 0x100000000 && vtable_ptr < 0x800000000000 {
            let vtable_addr = Address::new(vtable_ptr);
            layout.set_vtable(0, vtable_addr);

            // Analyze vtable
            if let Ok(vtable) = self.vtable_analyzer.analyze_vtable(vtable_addr, class_name) {
                for entry in &vtable.entries {
                    let method = CppVirtualMethod::new(
                        entry.function_name.as_deref().unwrap_or(&format!("vfunc{}", entry.index)),
                        entry.index,
                        entry.function_address
                    );
                    layout.add_virtual_method(method);
                }

                if let Some(rtti) = vtable.rtti_address {
                    layout.rtti_address = Some(rtti);
                }
            }
        }

        // Analyze member fields
        let start_offset = if layout.is_polymorphic { 8 } else { 0 };
        self.analyze_members(&mut layout, instance_addr, start_offset, estimated_size)?;

        layout.size = estimated_size;
        layout.find_padding();

        self.layouts.insert(class_name.to_string(), layout.clone());
        Ok(layout)
    }

    /// Analyze members by reading and classifying bytes
    fn analyze_members(&self, layout: &mut CppClassLayout, base_addr: Address, start_offset: usize, size: usize) -> Result<(), MemoryError> {
        let mut offset = start_offset;
        let mut field_index = 0;

        while offset < size {
            let addr = base_addr + offset as u64;
            let type_info = self.infer_type_at(addr)?;
            let field_size = type_info.size().max(1);

            let name = format!("field_{:X}", offset);
            let member = CppMember::new(&name, offset, type_info);
            layout.add_member(member);

            // Align to next field
            offset += field_size;
            let align = field_size.min(8);
            offset = (offset + align - 1) & !(align - 1);
            
            field_index += 1;
            if field_index > 1000 {
                break; // Safety limit
            }
        }

        Ok(())
    }

    /// Infer the type at a given address
    fn infer_type_at(&self, addr: Address) -> Result<TypeInfo, MemoryError> {
        let value = self.reader.read_u64(addr)?;

        // Check for pointer
        if value >= 0x100000000 && value < 0x800000000000 {
            // Check if it points to something readable
            if self.reader.read_u64(Address::new(value)).is_ok() {
                return Ok(TypeInfo::Pointer(Box::new(TypeInfo::Unknown)));
            }
        }

        // Check for small values (likely int/float)
        if value <= 0xFFFFFFFF {
            let value32 = value as u32;
            
            // Could be float?
            let as_float = f32::from_bits(value32);
            if as_float.is_finite() && as_float.abs() < 1e10 && as_float.abs() > 1e-10 {
                return Ok(TypeInfo::Primitive(PrimitiveType::F32));
            }
            
            return Ok(TypeInfo::Primitive(PrimitiveType::U32));
        }

        // Could be double?
        let as_double = f64::from_bits(value);
        if as_double.is_finite() && as_double.abs() < 1e20 && as_double.abs() > 1e-20 {
            return Ok(TypeInfo::Primitive(PrimitiveType::F64));
        }

        Ok(TypeInfo::Primitive(PrimitiveType::U64))
    }

    /// Get a reconstructed layout
    pub fn get_layout(&self, name: &str) -> Option<&CppClassLayout> {
        self.layouts.get(name)
    }

    /// Get all layouts
    pub fn layouts(&self) -> &HashMap<String, CppClassLayout> {
        &self.layouts
    }

    /// Compare two instances of the same class to refine the layout
    pub fn refine_from_instances(&mut self, instances: &[Address], class_name: &str) -> Result<CppClassLayout, MemoryError> {
        if instances.is_empty() {
            return Err(MemoryError::InvalidSize(0));
        }

        // Start with first instance
        let mut layout = self.reconstruct_from_instance(instances[0], class_name, 256)?;

        // Compare with other instances to find constant vs variable fields
        for instance in instances.iter().skip(1) {
            for member in &mut layout.members {
                let addr1 = instances[0] + member.offset as u64;
                let addr2 = *instance + member.offset as u64;

                let val1 = self.reader.read_u64(addr1)?;
                let val2 = self.reader.read_u64(addr2)?;

                if val1 == val2 {
                    // Likely a constant or vtable pointer
                    member.set_metadata("constant", "true");
                }
            }
        }

        Ok(layout)
    }
}

impl CppMember {
    fn set_metadata(&mut self, _key: &str, _value: &str) {
        // Store in type_info metadata if needed
    }
}

/// Builder for C++ class layouts
pub struct CppLayoutBuilder {
    layout: CppClassLayout,
}

impl CppLayoutBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            layout: CppClassLayout::new(name),
        }
    }

    pub fn size(mut self, size: usize) -> Self {
        self.layout.size = size;
        self
    }

    pub fn alignment(mut self, alignment: usize) -> Self {
        self.layout.alignment = alignment;
        self
    }

    pub fn add_base(mut self, name: &str, offset: usize) -> Self {
        self.layout.add_base(CppBaseClass::new(name, offset));
        self
    }

    pub fn add_virtual_base(mut self, name: &str, offset: usize) -> Self {
        self.layout.add_base(CppBaseClass::new(name, offset).with_virtual());
        self
    }

    pub fn add_member(mut self, name: &str, offset: usize, type_info: TypeInfo) -> Self {
        self.layout.add_member(CppMember::new(name, offset, type_info));
        self
    }

    pub fn add_private_member(mut self, name: &str, offset: usize, type_info: TypeInfo) -> Self {
        self.layout.add_member(CppMember::new(name, offset, type_info).with_access(AccessSpecifier::Private));
        self
    }

    pub fn add_protected_member(mut self, name: &str, offset: usize, type_info: TypeInfo) -> Self {
        self.layout.add_member(CppMember::new(name, offset, type_info).with_access(AccessSpecifier::Protected));
        self
    }

    pub fn vtable(mut self, offset: usize, address: Address) -> Self {
        self.layout.set_vtable(offset, address);
        self
    }

    pub fn add_virtual_method(mut self, name: &str, index: usize, address: Address) -> Self {
        self.layout.add_virtual_method(CppVirtualMethod::new(name, index, address));
        self
    }

    pub fn abstract_class(mut self) -> Self {
        self.layout.is_abstract = true;
        self
    }

    pub fn build(mut self) -> CppClassLayout {
        self.layout.find_padding();
        self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_layout_builder() {
        let layout = CppLayoutBuilder::new("TestClass")
            .size(32)
            .alignment(8)
            .add_member("x", 0, TypeInfo::Primitive(PrimitiveType::I32))
            .add_member("y", 4, TypeInfo::Primitive(PrimitiveType::I32))
            .add_member("ptr", 8, TypeInfo::Pointer(Box::new(TypeInfo::Unknown)))
            .build();

        assert_eq!(layout.name, "TestClass");
        assert_eq!(layout.size, 32);
        assert_eq!(layout.members.len(), 3);
    }

    #[test]
    fn test_cpp_member_display() {
        let member = CppMember::new("count", 16, TypeInfo::Primitive(PrimitiveType::U64))
            .with_access(AccessSpecifier::Private);
        
        let display = format!("{}", member);
        assert!(display.contains("private"));
        assert!(display.contains("count"));
        assert!(display.contains("0x10"));
    }
}
