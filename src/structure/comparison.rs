// Tue Jan 15 2026 - Alex

use crate::structure::{StructureLayout, Field, TypeInfo};
use crate::structure::cpp_layout::CppClassLayout;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents a difference between two structures
#[derive(Debug, Clone)]
pub enum StructureDifference {
    /// Field added in second structure
    FieldAdded {
        field_name: String,
        offset: usize,
        type_info: TypeInfo,
    },
    /// Field removed in second structure
    FieldRemoved {
        field_name: String,
        offset: usize,
        type_info: TypeInfo,
    },
    /// Field moved to different offset
    FieldMoved {
        field_name: String,
        old_offset: usize,
        new_offset: usize,
    },
    /// Field type changed
    TypeChanged {
        field_name: String,
        offset: usize,
        old_type: TypeInfo,
        new_type: TypeInfo,
    },
    /// Field size changed
    SizeChanged {
        field_name: String,
        offset: usize,
        old_size: usize,
        new_size: usize,
    },
    /// Field renamed (same offset, different name)
    FieldRenamed {
        old_name: String,
        new_name: String,
        offset: usize,
    },
    /// Structure size changed
    StructureSizeChanged {
        old_size: usize,
        new_size: usize,
    },
    /// Structure alignment changed
    AlignmentChanged {
        old_alignment: usize,
        new_alignment: usize,
    },
    /// Padding changed
    PaddingChanged {
        offset: usize,
        old_padding: usize,
        new_padding: usize,
    },
}

impl StructureDifference {
    pub fn severity(&self) -> DifferenceSeverity {
        match self {
            StructureDifference::StructureSizeChanged { .. } => DifferenceSeverity::Breaking,
            StructureDifference::FieldRemoved { .. } => DifferenceSeverity::Breaking,
            StructureDifference::FieldMoved { .. } => DifferenceSeverity::Breaking,
            StructureDifference::TypeChanged { .. } => DifferenceSeverity::Breaking,
            StructureDifference::SizeChanged { .. } => DifferenceSeverity::Breaking,
            StructureDifference::AlignmentChanged { .. } => DifferenceSeverity::Moderate,
            StructureDifference::FieldAdded { .. } => DifferenceSeverity::Minor,
            StructureDifference::FieldRenamed { .. } => DifferenceSeverity::Minor,
            StructureDifference::PaddingChanged { .. } => DifferenceSeverity::Informational,
        }
    }
}

impl fmt::Display for StructureDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructureDifference::FieldAdded { field_name, offset, type_info } => {
                write!(f, "Added: {} {} @ 0x{:X}", type_info, field_name, offset)
            }
            StructureDifference::FieldRemoved { field_name, offset, type_info } => {
                write!(f, "Removed: {} {} @ 0x{:X}", type_info, field_name, offset)
            }
            StructureDifference::FieldMoved { field_name, old_offset, new_offset } => {
                write!(f, "Moved: {} from 0x{:X} to 0x{:X}", field_name, old_offset, new_offset)
            }
            StructureDifference::TypeChanged { field_name, offset, old_type, new_type } => {
                write!(f, "Type changed: {} @ 0x{:X}: {} -> {}", field_name, offset, old_type, new_type)
            }
            StructureDifference::SizeChanged { field_name, offset, old_size, new_size } => {
                write!(f, "Size changed: {} @ 0x{:X}: {} -> {} bytes", field_name, offset, old_size, new_size)
            }
            StructureDifference::FieldRenamed { old_name, new_name, offset } => {
                write!(f, "Renamed: {} -> {} @ 0x{:X}", old_name, new_name, offset)
            }
            StructureDifference::StructureSizeChanged { old_size, new_size } => {
                write!(f, "Structure size changed: {} -> {} bytes", old_size, new_size)
            }
            StructureDifference::AlignmentChanged { old_alignment, new_alignment } => {
                write!(f, "Alignment changed: {} -> {}", old_alignment, new_alignment)
            }
            StructureDifference::PaddingChanged { offset, old_padding, new_padding } => {
                write!(f, "Padding changed @ 0x{:X}: {} -> {} bytes", offset, old_padding, new_padding)
            }
        }
    }
}

/// Severity level of a difference
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DifferenceSeverity {
    Informational,
    Minor,
    Moderate,
    Breaking,
}

impl fmt::Display for DifferenceSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DifferenceSeverity::Informational => write!(f, "INFO"),
            DifferenceSeverity::Minor => write!(f, "MINOR"),
            DifferenceSeverity::Moderate => write!(f, "MODERATE"),
            DifferenceSeverity::Breaking => write!(f, "BREAKING"),
        }
    }
}

/// Result of comparing two structures
#[derive(Debug, Clone)]
pub struct StructureComparison {
    /// Name of the first structure
    pub name1: String,
    /// Name of the second structure
    pub name2: String,
    /// All differences found
    pub differences: Vec<StructureDifference>,
    /// Fields that match exactly
    pub matching_fields: Vec<String>,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f64,
    /// Whether the structures are ABI compatible
    pub is_abi_compatible: bool,
}

impl StructureComparison {
    pub fn new(name1: &str, name2: &str) -> Self {
        Self {
            name1: name1.to_string(),
            name2: name2.to_string(),
            differences: Vec::new(),
            matching_fields: Vec::new(),
            similarity: 1.0,
            is_abi_compatible: true,
        }
    }

    pub fn add_difference(&mut self, diff: StructureDifference) {
        if diff.severity() >= DifferenceSeverity::Moderate {
            self.is_abi_compatible = false;
        }
        self.differences.push(diff);
    }

    pub fn add_match(&mut self, field_name: &str) {
        self.matching_fields.push(field_name.to_string());
    }

    pub fn calculate_similarity(&mut self) {
        let total = self.differences.len() + self.matching_fields.len();
        if total == 0 {
            self.similarity = 1.0;
        } else {
            self.similarity = self.matching_fields.len() as f64 / total as f64;
        }
    }

    pub fn has_breaking_changes(&self) -> bool {
        self.differences.iter().any(|d| d.severity() == DifferenceSeverity::Breaking)
    }

    pub fn breaking_changes(&self) -> Vec<&StructureDifference> {
        self.differences.iter()
            .filter(|d| d.severity() == DifferenceSeverity::Breaking)
            .collect()
    }

    pub fn non_breaking_changes(&self) -> Vec<&StructureDifference> {
        self.differences.iter()
            .filter(|d| d.severity() < DifferenceSeverity::Breaking)
            .collect()
    }

    pub fn group_by_severity(&self) -> HashMap<DifferenceSeverity, Vec<&StructureDifference>> {
        let mut groups: HashMap<DifferenceSeverity, Vec<&StructureDifference>> = HashMap::new();
        for diff in &self.differences {
            groups.entry(diff.severity()).or_default().push(diff);
        }
        groups
    }
}

impl fmt::Display for StructureComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Comparison: {} vs {}", self.name1, self.name2)?;
        writeln!(f, "Similarity: {:.1}%", self.similarity * 100.0)?;
        writeln!(f, "ABI Compatible: {}", if self.is_abi_compatible { "Yes" } else { "No" })?;
        writeln!(f, "Matching fields: {}", self.matching_fields.len())?;
        writeln!(f, "Differences: {}", self.differences.len())?;

        if !self.differences.is_empty() {
            writeln!(f, "\nDifferences:")?;
            for diff in &self.differences {
                writeln!(f, "  [{}] {}", diff.severity(), diff)?;
            }
        }

        Ok(())
    }
}

/// Compares two structures and finds differences
pub struct StructureComparator {
    /// Whether to consider field names when comparing
    compare_names: bool,
    /// Whether to consider field types when comparing
    compare_types: bool,
    /// Whether to detect renames (fields at same offset with different names)
    detect_renames: bool,
    /// Tolerance for considering fields "close" (for fuzzy matching)
    offset_tolerance: usize,
}

impl StructureComparator {
    pub fn new() -> Self {
        Self {
            compare_names: true,
            compare_types: true,
            detect_renames: true,
            offset_tolerance: 0,
        }
    }

    pub fn with_name_comparison(mut self, enabled: bool) -> Self {
        self.compare_names = enabled;
        self
    }

    pub fn with_type_comparison(mut self, enabled: bool) -> Self {
        self.compare_types = enabled;
        self
    }

    pub fn with_rename_detection(mut self, enabled: bool) -> Self {
        self.detect_renames = enabled;
        self
    }

    pub fn with_offset_tolerance(mut self, tolerance: usize) -> Self {
        self.offset_tolerance = tolerance;
        self
    }

    /// Compare two StructureLayout instances
    pub fn compare_layouts(&self, layout1: &StructureLayout, layout2: &StructureLayout) -> StructureComparison {
        let mut comparison = StructureComparison::new(layout1.name(), layout2.name());

        // Check structure size
        if layout1.size().as_u64() != layout2.size().as_u64() {
            comparison.add_difference(StructureDifference::StructureSizeChanged {
                old_size: layout1.size().as_u64() as usize,
                new_size: layout2.size().as_u64() as usize,
            });
        }

        // Check alignment
        if layout1.alignment().as_usize() != layout2.alignment().as_usize() {
            comparison.add_difference(StructureDifference::AlignmentChanged {
                old_alignment: layout1.alignment().as_usize(),
                new_alignment: layout2.alignment().as_usize(),
            });
        }

        // Build maps of fields by offset and name
        let fields1: HashMap<usize, &Field> = layout1.fields().iter()
            .map(|f| (f.offset().as_u64() as usize, f))
            .collect();
        let fields2: HashMap<usize, &Field> = layout2.fields().iter()
            .map(|f| (f.offset().as_u64() as usize, f))
            .collect();

        let names1: HashMap<&str, &Field> = layout1.fields().iter()
            .map(|f| (f.name(), f))
            .collect();
        let names2: HashMap<&str, &Field> = layout2.fields().iter()
            .map(|f| (f.name(), f))
            .collect();

        let offsets1: HashSet<usize> = fields1.keys().copied().collect();
        let offsets2: HashSet<usize> = fields2.keys().copied().collect();

        // Check each field in first layout
        for field1 in layout1.fields() {
            let offset1 = field1.offset().as_u64() as usize;
            let name1 = field1.name();

            if let Some(field2) = fields2.get(&offset1) {
                // Field exists at same offset
                if self.compare_names && field1.name() != field2.name() {
                    // Possible rename
                    if self.detect_renames {
                        comparison.add_difference(StructureDifference::FieldRenamed {
                            old_name: name1.to_string(),
                            new_name: field2.name().to_string(),
                            offset: offset1,
                        });
                    }
                }

                if self.compare_types && field1.type_info() != field2.type_info() {
                    comparison.add_difference(StructureDifference::TypeChanged {
                        field_name: name1.to_string(),
                        offset: offset1,
                        old_type: field1.type_info().clone(),
                        new_type: field2.type_info().clone(),
                    });
                } else if field1.size().as_u64() != field2.size().as_u64() {
                    comparison.add_difference(StructureDifference::SizeChanged {
                        field_name: name1.to_string(),
                        offset: offset1,
                        old_size: field1.size().as_u64() as usize,
                        new_size: field2.size().as_u64() as usize,
                    });
                } else {
                    comparison.add_match(name1);
                }
            } else if let Some(field2) = names2.get(name1) {
                // Field moved (same name, different offset)
                comparison.add_difference(StructureDifference::FieldMoved {
                    field_name: name1.to_string(),
                    old_offset: offset1,
                    new_offset: field2.offset().as_u64() as usize,
                });
            } else {
                // Field removed
                comparison.add_difference(StructureDifference::FieldRemoved {
                    field_name: name1.to_string(),
                    offset: offset1,
                    type_info: field1.type_info().clone(),
                });
            }
        }

        // Check for new fields in second layout
        for field2 in layout2.fields() {
            let offset2 = field2.offset().as_u64() as usize;
            let name2 = field2.name();

            if !fields1.contains_key(&offset2) && !names1.contains_key(name2) {
                comparison.add_difference(StructureDifference::FieldAdded {
                    field_name: name2.to_string(),
                    offset: offset2,
                    type_info: field2.type_info().clone(),
                });
            }
        }

        comparison.calculate_similarity();
        comparison
    }

    /// Compare two C++ class layouts
    pub fn compare_cpp_layouts(&self, layout1: &CppClassLayout, layout2: &CppClassLayout) -> StructureComparison {
        let mut comparison = StructureComparison::new(&layout1.name, &layout2.name);

        // Check structure size
        if layout1.size != layout2.size {
            comparison.add_difference(StructureDifference::StructureSizeChanged {
                old_size: layout1.size,
                new_size: layout2.size,
            });
        }

        // Check alignment
        if layout1.alignment != layout2.alignment {
            comparison.add_difference(StructureDifference::AlignmentChanged {
                old_alignment: layout1.alignment,
                new_alignment: layout2.alignment,
            });
        }

        // Build maps
        let members1: HashMap<usize, _> = layout1.members.iter()
            .map(|m| (m.offset, m))
            .collect();
        let members2: HashMap<usize, _> = layout2.members.iter()
            .map(|m| (m.offset, m))
            .collect();

        let names1: HashMap<&str, _> = layout1.members.iter()
            .map(|m| (m.name.as_str(), m))
            .collect();
        let names2: HashMap<&str, _> = layout2.members.iter()
            .map(|m| (m.name.as_str(), m))
            .collect();

        // Compare members
        for member1 in &layout1.members {
            if let Some(member2) = members2.get(&member1.offset) {
                if self.compare_names && member1.name != member2.name {
                    comparison.add_difference(StructureDifference::FieldRenamed {
                        old_name: member1.name.clone(),
                        new_name: member2.name.clone(),
                        offset: member1.offset,
                    });
                }

                if self.compare_types && member1.type_info != member2.type_info {
                    comparison.add_difference(StructureDifference::TypeChanged {
                        field_name: member1.name.clone(),
                        offset: member1.offset,
                        old_type: member1.type_info.clone(),
                        new_type: member2.type_info.clone(),
                    });
                } else if member1.size != member2.size {
                    comparison.add_difference(StructureDifference::SizeChanged {
                        field_name: member1.name.clone(),
                        offset: member1.offset,
                        old_size: member1.size,
                        new_size: member2.size,
                    });
                } else {
                    comparison.add_match(&member1.name);
                }
            } else if let Some(member2) = names2.get(member1.name.as_str()) {
                comparison.add_difference(StructureDifference::FieldMoved {
                    field_name: member1.name.clone(),
                    old_offset: member1.offset,
                    new_offset: member2.offset,
                });
            } else {
                comparison.add_difference(StructureDifference::FieldRemoved {
                    field_name: member1.name.clone(),
                    offset: member1.offset,
                    type_info: member1.type_info.clone(),
                });
            }
        }

        // Check for new members
        for member2 in &layout2.members {
            if !members1.contains_key(&member2.offset) && !names1.contains_key(member2.name.as_str()) {
                comparison.add_difference(StructureDifference::FieldAdded {
                    field_name: member2.name.clone(),
                    offset: member2.offset,
                    type_info: member2.type_info.clone(),
                });
            }
        }

        // Check padding differences
        for (offset1, size1) in &layout1.padding {
            let matching_padding = layout2.padding.iter()
                .find(|(o, _)| *o == *offset1);
            
            if let Some((_, size2)) = matching_padding {
                if size1 != size2 {
                    comparison.add_difference(StructureDifference::PaddingChanged {
                        offset: *offset1,
                        old_padding: *size1,
                        new_padding: *size2,
                    });
                }
            }
        }

        comparison.calculate_similarity();
        comparison
    }
}

impl Default for StructureComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates migration information between struct versions
#[derive(Debug, Clone)]
pub struct MigrationInfo {
    pub source_version: String,
    pub target_version: String,
    pub field_mappings: Vec<FieldMapping>,
    pub requires_resize: bool,
    pub new_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FieldMapping {
    pub source_field: String,
    pub source_offset: usize,
    pub target_field: String,
    pub target_offset: usize,
    pub needs_conversion: bool,
    pub conversion_note: Option<String>,
}

impl MigrationInfo {
    pub fn from_comparison(comparison: &StructureComparison) -> Self {
        let mut info = MigrationInfo {
            source_version: comparison.name1.clone(),
            target_version: comparison.name2.clone(),
            field_mappings: Vec::new(),
            requires_resize: false,
            new_size: None,
        };

        // Generate mappings from differences
        for diff in &comparison.differences {
            match diff {
                StructureDifference::StructureSizeChanged { new_size, .. } => {
                    info.requires_resize = true;
                    info.new_size = Some(*new_size);
                }
                StructureDifference::FieldMoved { field_name, old_offset, new_offset } => {
                    info.field_mappings.push(FieldMapping {
                        source_field: field_name.clone(),
                        source_offset: *old_offset,
                        target_field: field_name.clone(),
                        target_offset: *new_offset,
                        needs_conversion: false,
                        conversion_note: None,
                    });
                }
                StructureDifference::FieldRenamed { old_name, new_name, offset } => {
                    info.field_mappings.push(FieldMapping {
                        source_field: old_name.clone(),
                        source_offset: *offset,
                        target_field: new_name.clone(),
                        target_offset: *offset,
                        needs_conversion: false,
                        conversion_note: Some("Renamed".to_string()),
                    });
                }
                StructureDifference::TypeChanged { field_name, offset, old_type, new_type } => {
                    info.field_mappings.push(FieldMapping {
                        source_field: field_name.clone(),
                        source_offset: *offset,
                        target_field: field_name.clone(),
                        target_offset: *offset,
                        needs_conversion: true,
                        conversion_note: Some(format!("{} -> {}", old_type, new_type)),
                    });
                }
                _ => {}
            }
        }

        // Add matching fields
        for field_name in &comparison.matching_fields {
            // Would need original offset info - simplified for now
            info.field_mappings.push(FieldMapping {
                source_field: field_name.clone(),
                source_offset: 0,
                target_field: field_name.clone(),
                target_offset: 0,
                needs_conversion: false,
                conversion_note: None,
            });
        }

        info
    }

    pub fn has_field_movements(&self) -> bool {
        self.field_mappings.iter().any(|m| m.source_offset != m.target_offset)
    }

    pub fn has_conversions(&self) -> bool {
        self.field_mappings.iter().any(|m| m.needs_conversion)
    }
}

impl fmt::Display for MigrationInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Migration: {} -> {}", self.source_version, self.target_version)?;
        if self.requires_resize {
            writeln!(f, "Requires resize to {} bytes", self.new_size.unwrap_or(0))?;
        }
        writeln!(f, "Field mappings:")?;
        for mapping in &self.field_mappings {
            write!(f, "  {} @ 0x{:X} -> {} @ 0x{:X}", 
                mapping.source_field, mapping.source_offset,
                mapping.target_field, mapping.target_offset)?;
            if let Some(ref note) = mapping.conversion_note {
                write!(f, " ({})", note)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::type_info::PrimitiveType;

    #[test]
    fn test_structure_comparison() {
        let comparator = StructureComparator::new();
        
        let mut layout1 = StructureLayout::new("TestStruct".to_string());
        layout1.add_field(Field::new("x".to_string(), Offset::new(0), TypeInfo::Primitive(PrimitiveType::I32)));
        layout1.add_field(Field::new("y".to_string(), Offset::new(4), TypeInfo::Primitive(PrimitiveType::I32)));

        let mut layout2 = StructureLayout::new("TestStruct".to_string());
        layout2.add_field(Field::new("x".to_string(), Offset::new(0), TypeInfo::Primitive(PrimitiveType::I32)));
        layout2.add_field(Field::new("z".to_string(), Offset::new(4), TypeInfo::Primitive(PrimitiveType::I32)));

        let comparison = comparator.compare_layouts(&layout1, &layout2);
        
        assert!(!comparison.differences.is_empty());
        assert!(comparison.similarity < 1.0);
    }

    #[test]
    fn test_difference_severity() {
        let breaking = StructureDifference::FieldRemoved {
            field_name: "test".to_string(),
            offset: 0,
            type_info: TypeInfo::Unknown,
        };
        assert_eq!(breaking.severity(), DifferenceSeverity::Breaking);

        let minor = StructureDifference::FieldAdded {
            field_name: "test".to_string(),
            offset: 0,
            type_info: TypeInfo::Unknown,
        };
        assert_eq!(minor.severity(), DifferenceSeverity::Minor);
    }
}
