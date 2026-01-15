// Tue Jan 15 2026 - Alex

pub mod analyzer;
pub mod layout;
pub mod field;
pub mod type_info;
pub mod builder;
pub mod error;
pub mod cache;
pub mod offset;
pub mod alignment;
pub mod size;
pub mod member;
pub mod traversal;
pub mod inference;
pub mod validator;
pub mod serializer;
pub mod vtable;
pub mod inheritance;
pub mod cpp_layout;
pub mod comparison;

pub use analyzer::StructureAnalyzer;
pub use layout::StructureLayout;
pub use field::Field;
pub use type_info::{TypeInfo, PrimitiveType};
pub use builder::StructureBuilder;
pub use error::StructureError;
pub use offset::Offset;
pub use alignment::Alignment;
pub use size::Size;
pub use member::Member;
pub use inference::TypeInference;
pub use validator::StructureValidator;
pub use serializer::SerializableLayout;
pub use vtable::{VTable, VTableEntry, VTableAnalyzer, VTableComparison, VTableDifference, InheritanceInfo, VTableBuilder};
pub use inheritance::{ClassNode, ClassHierarchy, InheritanceDetector, HierarchyStats};
pub use cpp_layout::{CppClassLayout, CppMember, CppVirtualMethod, CppBaseClass, AccessSpecifier, CppLayoutReconstructor, CppLayoutBuilder};
pub use comparison::{StructureComparison, StructureComparator, StructureDifference, DifferenceSeverity, MigrationInfo};
