// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, FieldOffset};
use std::collections::{HashMap, HashSet};

pub struct DiffGenerator {
    show_added: bool,
    show_removed: bool,
    show_changed: bool,
    include_details: bool,
    threshold_percent: f64,
}

#[derive(Debug, Clone)]
pub struct OffsetDiff {
    pub old_version: String,
    pub new_version: String,
    pub old_target: String,
    pub new_target: String,
    pub function_diff: FunctionDiff,
    pub structure_diff: StructureDiff,
    pub class_diff: ClassDiff,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionDiff {
    pub added: Vec<FunctionChange>,
    pub removed: Vec<FunctionChange>,
    pub changed: Vec<FunctionChange>,
    pub unchanged: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionChange {
    pub name: String,
    pub old_address: Option<u64>,
    pub new_address: Option<u64>,
    pub old_confidence: Option<f64>,
    pub new_confidence: Option<f64>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Default)]
pub struct StructureDiff {
    pub added: Vec<StructureChange>,
    pub removed: Vec<StructureChange>,
    pub changed: Vec<StructureChange>,
    pub unchanged: usize,
}

#[derive(Debug, Clone)]
pub struct StructureChange {
    pub name: String,
    pub old_size: Option<usize>,
    pub new_size: Option<usize>,
    pub field_changes: Vec<FieldChange>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone)]
pub struct FieldChange {
    pub field_name: String,
    pub old_offset: Option<usize>,
    pub new_offset: Option<usize>,
    pub old_type: Option<String>,
    pub new_type: Option<String>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Default)]
pub struct ClassDiff {
    pub added: Vec<ClassChange>,
    pub removed: Vec<ClassChange>,
    pub changed: Vec<ClassChange>,
    pub unchanged: usize,
}

#[derive(Debug, Clone)]
pub struct ClassChange {
    pub name: String,
    pub old_vtable: Option<u64>,
    pub new_vtable: Option<u64>,
    pub old_size: Option<usize>,
    pub new_size: Option<usize>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Removed,
    AddressChanged,
    SizeChanged,
    TypeChanged,
    MultipleChanges,
    Unchanged,
}

#[derive(Debug, Clone, Default)]
pub struct DiffSummary {
    pub functions_added: usize,
    pub functions_removed: usize,
    pub functions_changed: usize,
    pub functions_unchanged: usize,
    pub structures_added: usize,
    pub structures_removed: usize,
    pub structures_changed: usize,
    pub structures_unchanged: usize,
    pub classes_added: usize,
    pub classes_removed: usize,
    pub classes_changed: usize,
    pub classes_unchanged: usize,
    pub total_changes: usize,
    pub change_percentage: f64,
}

impl DiffGenerator {
    pub fn new() -> Self {
        Self {
            show_added: true,
            show_removed: true,
            show_changed: true,
            include_details: true,
            threshold_percent: 0.0,
        }
    }

    pub fn with_added(mut self, show: bool) -> Self {
        self.show_added = show;
        self
    }

    pub fn with_removed(mut self, show: bool) -> Self {
        self.show_removed = show;
        self
    }

    pub fn with_changed(mut self, show: bool) -> Self {
        self.show_changed = show;
        self
    }

    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }

    pub fn with_threshold(mut self, percent: f64) -> Self {
        self.threshold_percent = percent;
        self
    }

    pub fn generate(&self, old: &OffsetOutput, new: &OffsetOutput) -> OffsetDiff {
        let function_diff = self.diff_functions(&old.functions, &new.functions);
        let structure_diff = self.diff_structures(&old.structure_offsets, &new.structure_offsets);
        let class_diff = self.diff_classes(&old.classes, &new.classes);

        let summary = DiffSummary {
            functions_added: function_diff.added.len(),
            functions_removed: function_diff.removed.len(),
            functions_changed: function_diff.changed.len(),
            functions_unchanged: function_diff.unchanged,
            structures_added: structure_diff.added.len(),
            structures_removed: structure_diff.removed.len(),
            structures_changed: structure_diff.changed.len(),
            structures_unchanged: structure_diff.unchanged,
            classes_added: class_diff.added.len(),
            classes_removed: class_diff.removed.len(),
            classes_changed: class_diff.changed.len(),
            classes_unchanged: class_diff.unchanged,
            total_changes: function_diff.added.len() + function_diff.removed.len() + function_diff.changed.len() +
                          structure_diff.added.len() + structure_diff.removed.len() + structure_diff.changed.len() +
                          class_diff.added.len() + class_diff.removed.len() + class_diff.changed.len(),
            change_percentage: 0.0,
        };

        OffsetDiff {
            old_version: old.version.clone(),
            new_version: new.version.clone(),
            old_target: old.target.name.clone(),
            new_target: new.target.name.clone(),
            function_diff,
            structure_diff,
            class_diff,
            summary,
        }
    }

    fn diff_functions(&self, old: &HashMap<String, FunctionOffset>, new: &HashMap<String, FunctionOffset>) -> FunctionDiff {
        let mut diff = FunctionDiff::default();

        let old_names: HashSet<_> = old.keys().collect();
        let new_names: HashSet<_> = new.keys().collect();

        for name in new_names.difference(&old_names) {
            if self.show_added {
                let func = &new[*name];
                diff.added.push(FunctionChange {
                    name: (*name).clone(),
                    old_address: None,
                    new_address: Some(func.address),
                    old_confidence: None,
                    new_confidence: Some(func.confidence),
                    change_type: ChangeType::Added,
                });
            }
        }

        for name in old_names.difference(&new_names) {
            if self.show_removed {
                let func = &old[*name];
                diff.removed.push(FunctionChange {
                    name: (*name).clone(),
                    old_address: Some(func.address),
                    new_address: None,
                    old_confidence: Some(func.confidence),
                    new_confidence: None,
                    change_type: ChangeType::Removed,
                });
            }
        }

        for name in old_names.intersection(&new_names) {
            let old_func = &old[*name];
            let new_func = &new[*name];

            if old_func.address != new_func.address {
                if self.show_changed {
                    diff.changed.push(FunctionChange {
                        name: (*name).clone(),
                        old_address: Some(old_func.address),
                        new_address: Some(new_func.address),
                        old_confidence: Some(old_func.confidence),
                        new_confidence: Some(new_func.confidence),
                        change_type: ChangeType::AddressChanged,
                    });
                }
            } else {
                diff.unchanged += 1;
            }
        }

        diff
    }

    fn diff_structures(&self, old: &HashMap<String, StructureOffsets>, new: &HashMap<String, StructureOffsets>) -> StructureDiff {
        let mut diff = StructureDiff::default();

        let old_names: HashSet<_> = old.keys().collect();
        let new_names: HashSet<_> = new.keys().collect();

        for name in new_names.difference(&old_names) {
            if self.show_added {
                let structure = &new[*name];
                diff.added.push(StructureChange {
                    name: (*name).clone(),
                    old_size: None,
                    new_size: Some(structure.size),
                    field_changes: Vec::new(),
                    change_type: ChangeType::Added,
                });
            }
        }

        for name in old_names.difference(&new_names) {
            if self.show_removed {
                let structure = &old[*name];
                diff.removed.push(StructureChange {
                    name: (*name).clone(),
                    old_size: Some(structure.size),
                    new_size: None,
                    field_changes: Vec::new(),
                    change_type: ChangeType::Removed,
                });
            }
        }

        for name in old_names.intersection(&new_names) {
            let old_struct = &old[*name];
            let new_struct = &new[*name];

            let field_changes = if self.include_details {
                self.diff_fields(&old_struct.fields, &new_struct.fields)
            } else {
                Vec::new()
            };

            let has_changes = old_struct.size != new_struct.size || !field_changes.is_empty();

            if has_changes {
                if self.show_changed {
                    let change_type = if old_struct.size != new_struct.size && !field_changes.is_empty() {
                        ChangeType::MultipleChanges
                    } else if old_struct.size != new_struct.size {
                        ChangeType::SizeChanged
                    } else {
                        ChangeType::MultipleChanges
                    };

                    diff.changed.push(StructureChange {
                        name: (*name).clone(),
                        old_size: Some(old_struct.size),
                        new_size: Some(new_struct.size),
                        field_changes,
                        change_type,
                    });
                }
            } else {
                diff.unchanged += 1;
            }
        }

        diff
    }

    fn diff_fields(&self, old: &HashMap<String, FieldOffset>, new: &HashMap<String, FieldOffset>) -> Vec<FieldChange> {
        let mut changes = Vec::new();

        let old_names: HashSet<_> = old.keys().collect();
        let new_names: HashSet<_> = new.keys().collect();

        for name in new_names.difference(&old_names) {
            let field = &new[*name];
            changes.push(FieldChange {
                field_name: (*name).clone(),
                old_offset: None,
                new_offset: Some(field.offset),
                old_type: None,
                new_type: Some(field.field_type.clone()),
                change_type: ChangeType::Added,
            });
        }

        for name in old_names.difference(&new_names) {
            let field = &old[*name];
            changes.push(FieldChange {
                field_name: (*name).clone(),
                old_offset: Some(field.offset),
                new_offset: None,
                old_type: Some(field.field_type.clone()),
                new_type: None,
                change_type: ChangeType::Removed,
            });
        }

        for name in old_names.intersection(&new_names) {
            let old_field = &old[*name];
            let new_field = &new[*name];

            let offset_changed = old_field.offset != new_field.offset;
            let type_changed = old_field.field_type != new_field.field_type;

            if offset_changed || type_changed {
                let change_type = if offset_changed && type_changed {
                    ChangeType::MultipleChanges
                } else if offset_changed {
                    ChangeType::AddressChanged
                } else {
                    ChangeType::TypeChanged
                };

                changes.push(FieldChange {
                    field_name: (*name).clone(),
                    old_offset: Some(old_field.offset),
                    new_offset: Some(new_field.offset),
                    old_type: Some(old_field.field_type.clone()),
                    new_type: Some(new_field.field_type.clone()),
                    change_type,
                });
            }
        }

        changes
    }

    fn diff_classes(&self, old: &[ClassOffset], new: &[ClassOffset]) -> ClassDiff {
        let mut diff = ClassDiff::default();

        let old_map: HashMap<_, _> = old.iter().map(|c| (c.name.clone(), c)).collect();
        let new_map: HashMap<_, _> = new.iter().map(|c| (c.name.clone(), c)).collect();

        let old_names: HashSet<_> = old_map.keys().collect();
        let new_names: HashSet<_> = new_map.keys().collect();

        for name in new_names.difference(&old_names) {
            if self.show_added {
                let class = new_map[*name];
                diff.added.push(ClassChange {
                    name: (*name).clone(),
                    old_vtable: None,
                    new_vtable: class.vtable_address,
                    old_size: None,
                    new_size: Some(class.size),
                    change_type: ChangeType::Added,
                });
            }
        }

        for name in old_names.difference(&new_names) {
            if self.show_removed {
                let class = old_map[*name];
                diff.removed.push(ClassChange {
                    name: (*name).clone(),
                    old_vtable: class.vtable_address,
                    new_vtable: None,
                    old_size: Some(class.size),
                    new_size: None,
                    change_type: ChangeType::Removed,
                });
            }
        }

        for name in old_names.intersection(&new_names) {
            let old_class = old_map[*name];
            let new_class = new_map[*name];

            let vtable_changed = old_class.vtable_address != new_class.vtable_address;
            let size_changed = old_class.size != new_class.size;

            if vtable_changed || size_changed {
                if self.show_changed {
                    let change_type = if vtable_changed && size_changed {
                        ChangeType::MultipleChanges
                    } else if vtable_changed {
                        ChangeType::AddressChanged
                    } else {
                        ChangeType::SizeChanged
                    };

                    diff.changed.push(ClassChange {
                        name: (*name).clone(),
                        old_vtable: old_class.vtable_address,
                        new_vtable: new_class.vtable_address,
                        old_size: Some(old_class.size),
                        new_size: Some(new_class.size),
                        change_type,
                    });
                }
            } else {
                diff.unchanged += 1;
            }
        }

        diff
    }

    pub fn format_diff(&self, diff: &OffsetDiff) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Offset Diff: {} -> {} ===\n\n", diff.old_version, diff.new_version));

        output.push_str("Summary:\n");
        output.push_str(&format!("  Functions: +{} -{} ~{} ={}\n",
            diff.summary.functions_added,
            diff.summary.functions_removed,
            diff.summary.functions_changed,
            diff.summary.functions_unchanged));
        output.push_str(&format!("  Structures: +{} -{} ~{} ={}\n",
            diff.summary.structures_added,
            diff.summary.structures_removed,
            diff.summary.structures_changed,
            diff.summary.structures_unchanged));
        output.push_str(&format!("  Classes: +{} -{} ~{} ={}\n\n",
            diff.summary.classes_added,
            diff.summary.classes_removed,
            diff.summary.classes_changed,
            diff.summary.classes_unchanged));

        if !diff.function_diff.added.is_empty() {
            output.push_str("Added Functions:\n");
            for change in &diff.function_diff.added {
                output.push_str(&format!("  + {} @ 0x{:x}\n", change.name, change.new_address.unwrap_or(0)));
            }
            output.push('\n');
        }

        if !diff.function_diff.removed.is_empty() {
            output.push_str("Removed Functions:\n");
            for change in &diff.function_diff.removed {
                output.push_str(&format!("  - {} @ 0x{:x}\n", change.name, change.old_address.unwrap_or(0)));
            }
            output.push('\n');
        }

        if !diff.function_diff.changed.is_empty() {
            output.push_str("Changed Functions:\n");
            for change in &diff.function_diff.changed {
                output.push_str(&format!("  ~ {} 0x{:x} -> 0x{:x}\n",
                    change.name,
                    change.old_address.unwrap_or(0),
                    change.new_address.unwrap_or(0)));
            }
            output.push('\n');
        }

        if !diff.structure_diff.changed.is_empty() {
            output.push_str("Changed Structures:\n");
            for change in &diff.structure_diff.changed {
                output.push_str(&format!("  ~ {} (size: {} -> {})\n",
                    change.name,
                    change.old_size.unwrap_or(0),
                    change.new_size.unwrap_or(0)));

                for field in &change.field_changes {
                    match field.change_type {
                        ChangeType::Added => output.push_str(&format!("    + {} @ 0x{:x}\n",
                            field.field_name, field.new_offset.unwrap_or(0))),
                        ChangeType::Removed => output.push_str(&format!("    - {} @ 0x{:x}\n",
                            field.field_name, field.old_offset.unwrap_or(0))),
                        _ => output.push_str(&format!("    ~ {} 0x{:x} -> 0x{:x}\n",
                            field.field_name,
                            field.old_offset.unwrap_or(0),
                            field.new_offset.unwrap_or(0))),
                    }
                }
            }
        }

        output
    }
}

impl Default for DiffGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OffsetDiff {
    pub fn has_changes(&self) -> bool {
        self.summary.total_changes > 0
    }

    pub fn is_breaking(&self) -> bool {
        !self.function_diff.removed.is_empty() ||
        !self.function_diff.changed.is_empty() ||
        !self.structure_diff.removed.is_empty() ||
        !self.structure_diff.changed.is_empty()
    }

    pub fn get_breaking_changes(&self) -> Vec<String> {
        let mut changes = Vec::new();

        for change in &self.function_diff.removed {
            changes.push(format!("Function removed: {}", change.name));
        }
        for change in &self.function_diff.changed {
            changes.push(format!("Function address changed: {}", change.name));
        }
        for change in &self.structure_diff.removed {
            changes.push(format!("Structure removed: {}", change.name));
        }
        for change in &self.structure_diff.changed {
            changes.push(format!("Structure changed: {}", change.name));
        }

        changes
    }
}

pub fn generate_diff(old: &OffsetOutput, new: &OffsetOutput) -> OffsetDiff {
    DiffGenerator::new().generate(old, new)
}

pub fn format_diff(old: &OffsetOutput, new: &OffsetOutput) -> String {
    let diff = generate_diff(old, new);
    DiffGenerator::new().format_diff(&diff)
}
