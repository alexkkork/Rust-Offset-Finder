// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, PropertyOffset, MethodOffset, ConstantOffset, ConstantValue};
use crate::output::json::JsonSerializer;
use crate::output::report::{ReportGenerator, ReportFormat};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct OutputManager {
    output: Arc<RwLock<OffsetOutput>>,
    output_path: PathBuf,
    start_time: Instant,
    json_serializer: JsonSerializer,
    auto_save: bool,
    save_interval: usize,
    changes_since_save: usize,
    backup_enabled: bool,
    backup_count: usize,
}

impl OutputManager {
    pub fn new(target_name: &str, output_path: PathBuf) -> Self {
        Self {
            output: Arc::new(RwLock::new(OffsetOutput::new(target_name))),
            output_path,
            start_time: Instant::now(),
            json_serializer: JsonSerializer::new(),
            auto_save: true,
            save_interval: 100,
            changes_since_save: 0,
            backup_enabled: true,
            backup_count: 3,
        }
    }

    pub fn with_auto_save(mut self, enabled: bool) -> Self {
        self.auto_save = enabled;
        self
    }

    pub fn with_save_interval(mut self, interval: usize) -> Self {
        self.save_interval = interval;
        self
    }

    pub fn with_backup(mut self, enabled: bool) -> Self {
        self.backup_enabled = enabled;
        self
    }

    pub fn add_function(&self, name: &str, address: u64, confidence: f64, method: &str, category: &str) {
        let func = FunctionOffset::new(address, confidence, method)
            .with_category(category);

        let mut output = self.output.write().unwrap();
        output.add_function(name, func);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_function_with_signature(&self, name: &str, address: u64, confidence: f64, method: &str, category: &str, signature: &str) {
        let func = FunctionOffset::new(address, confidence, method)
            .with_category(category)
            .with_signature(signature);

        let mut output = self.output.write().unwrap();
        output.add_function(name, func);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_structure(&self, name: &str, size: usize, alignment: usize) {
        let structure = StructureOffsets::new(size, alignment);

        let mut output = self.output.write().unwrap();
        output.add_structure(name, structure);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_structure_field(&self, structure_name: &str, field_name: &str, offset: usize, size: usize, field_type: &str) {
        let mut output = self.output.write().unwrap();

        if let Some(structure) = output.structure_offsets.get_mut(structure_name) {
            structure.add_field(field_name, offset, size, field_type);
        } else {
            let mut structure = StructureOffsets::new(0, 8);
            structure.add_field(field_name, offset, size, field_type);
            output.add_structure(structure_name, structure);
        }

        drop(output);
        self.maybe_auto_save();
    }

    pub fn add_class(&self, name: &str, vtable: Option<u64>, size: usize, parent: Option<&str>) {
        let mut class = ClassOffset::new(name).with_size(size);
        if let Some(vt) = vtable {
            class = class.with_vtable(vt);
        }
        if let Some(p) = parent {
            class = class.with_parent(p);
        }

        let mut output = self.output.write().unwrap();
        output.add_class(class);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_property(&self, name: &str, class_name: &str, getter: Option<u64>, setter: Option<u64>, offset: Option<usize>, prop_type: &str) {
        let property = PropertyOffset {
            name: name.to_string(),
            class_name: class_name.to_string(),
            getter,
            setter,
            offset,
            property_type: prop_type.to_string(),
        };

        let mut output = self.output.write().unwrap();
        output.add_property(property);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_method(&self, name: &str, class_name: &str, address: u64, vtable_index: Option<usize>, is_virtual: bool, signature: Option<&str>) {
        let method = MethodOffset {
            name: name.to_string(),
            class_name: class_name.to_string(),
            address,
            vtable_index,
            is_virtual,
            signature: signature.map(|s| s.to_string()),
        };

        let mut output = self.output.write().unwrap();
        output.add_method(method);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_constant_integer(&self, name: &str, address: u64, value: i64, category: &str) {
        let constant = ConstantOffset {
            name: name.to_string(),
            address,
            value: ConstantValue::Integer(value),
            category: category.to_string(),
        };

        let mut output = self.output.write().unwrap();
        output.add_constant(constant);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_constant_float(&self, name: &str, address: u64, value: f64, category: &str) {
        let constant = ConstantOffset {
            name: name.to_string(),
            address,
            value: ConstantValue::Float(value),
            category: category.to_string(),
        };

        let mut output = self.output.write().unwrap();
        output.add_constant(constant);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_constant_string(&self, name: &str, address: u64, value: &str, category: &str) {
        let constant = ConstantOffset {
            name: name.to_string(),
            address,
            value: ConstantValue::String(value.to_string()),
            category: category.to_string(),
        };

        let mut output = self.output.write().unwrap();
        output.add_constant(constant);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn add_constant_address(&self, name: &str, address: u64, value: u64, category: &str) {
        let constant = ConstantOffset {
            name: name.to_string(),
            address,
            value: ConstantValue::Address(value),
            category: category.to_string(),
        };

        let mut output = self.output.write().unwrap();
        output.add_constant(constant);
        drop(output);

        self.maybe_auto_save();
    }

    pub fn set_target_info(&self, version: Option<&str>, hash: Option<&str>, base_address: u64) {
        let mut output = self.output.write().unwrap();
        if let Some(v) = version {
            output.set_target_version(v);
        }
        if let Some(h) = hash {
            output.set_target_hash(h);
        }
        output.set_base_address(base_address);
    }

    pub fn update_statistics(&self, memory_scanned: u64, patterns_matched: usize, symbols_resolved: usize, xrefs_analyzed: usize) {
        let mut output = self.output.write().unwrap();
        output.statistics.memory_scanned_bytes = memory_scanned;
        output.statistics.patterns_matched = patterns_matched;
        output.statistics.symbols_resolved = symbols_resolved;
        output.statistics.xrefs_analyzed = xrefs_analyzed;
        output.statistics.scan_duration_ms = self.start_time.elapsed().as_millis() as u64;
    }

    fn maybe_auto_save(&self) {
        if !self.auto_save {
            return;
        }

        let output = self.output.read().unwrap();
        let total_items = output.total_offsets();

        if total_items > 0 && total_items % self.save_interval == 0 {
            drop(output);
            let _ = self.save();
        }
    }

    pub fn save(&self) -> Result<(), OutputError> {
        let mut output = self.output.write().unwrap();
        output.compute_statistics();
        output.statistics.scan_duration_ms = self.start_time.elapsed().as_millis() as u64;

        if self.backup_enabled && self.output_path.exists() {
            self.rotate_backups()?;
        }

        self.json_serializer
            .serialize_to_file(&output, &self.output_path)
            .map_err(|e| OutputError::SaveError(e.to_string()))?;

        Ok(())
    }

    fn rotate_backups(&self) -> Result<(), OutputError> {
        for i in (1..self.backup_count).rev() {
            let old_path = self.backup_path(i);
            let new_path = self.backup_path(i + 1);
            if old_path.exists() {
                std::fs::rename(&old_path, &new_path)
                    .map_err(|e| OutputError::BackupError(e.to_string()))?;
            }
        }

        if self.output_path.exists() {
            let backup_path = self.backup_path(1);
            std::fs::copy(&self.output_path, &backup_path)
                .map_err(|e| OutputError::BackupError(e.to_string()))?;
        }

        Ok(())
    }

    fn backup_path(&self, index: usize) -> PathBuf {
        let stem = self.output_path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = self.output_path.extension().unwrap_or_default().to_string_lossy();
        let parent = self.output_path.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}.backup{}.{}", stem, index, ext))
    }

    pub fn load_existing(&self) -> Result<(), OutputError> {
        if !self.output_path.exists() {
            return Ok(());
        }

        let loaded = self.json_serializer
            .deserialize_from_file(&self.output_path)
            .map_err(|e| OutputError::LoadError(e.to_string()))?;

        let mut output = self.output.write().unwrap();
        *output = loaded;

        Ok(())
    }

    pub fn merge_with(&self, other_path: &Path) -> Result<(), OutputError> {
        let other = self.json_serializer
            .deserialize_from_file(other_path)
            .map_err(|e| OutputError::LoadError(e.to_string()))?;

        let mut output = self.output.write().unwrap();
        let merged = self.json_serializer.merge(&output, &other);
        *output = merged;

        Ok(())
    }

    pub fn generate_report(&self, format: ReportFormat, path: &Path) -> Result<(), OutputError> {
        let output = self.output.read().unwrap();
        let generator = ReportGenerator::new(format);
        generator.generate_to_file(&output, path)
            .map_err(|e| OutputError::ReportError(e.to_string()))?;
        Ok(())
    }

    pub fn get_summary(&self) -> OutputSummary {
        let output = self.output.read().unwrap();
        OutputSummary {
            total_functions: output.functions.len(),
            total_structures: output.structure_offsets.len(),
            total_classes: output.classes.len(),
            total_properties: output.properties.len(),
            total_methods: output.methods.len(),
            total_constants: output.constants.len(),
            total_offsets: output.total_offsets(),
            elapsed_ms: self.start_time.elapsed().as_millis() as u64,
        }
    }

    pub fn has_function(&self, name: &str) -> bool {
        let output = self.output.read().unwrap();
        output.functions.contains_key(name)
    }

    pub fn has_structure(&self, name: &str) -> bool {
        let output = self.output.read().unwrap();
        output.structure_offsets.contains_key(name)
    }

    pub fn get_function_address(&self, name: &str) -> Option<u64> {
        let output = self.output.read().unwrap();
        output.functions.get(name).map(|f| f.address)
    }

    pub fn get_all_function_names(&self) -> Vec<String> {
        let output = self.output.read().unwrap();
        output.functions.keys().cloned().collect()
    }

    pub fn get_functions_by_category(&self, category: &str) -> Vec<(String, u64)> {
        let output = self.output.read().unwrap();
        output.functions.iter()
            .filter(|(_, f)| f.category == category)
            .map(|(name, f)| (name.clone(), f.address))
            .collect()
    }

    pub fn get_low_confidence_functions(&self, threshold: f64) -> Vec<(String, f64)> {
        let output = self.output.read().unwrap();
        output.functions.iter()
            .filter(|(_, f)| f.confidence < threshold)
            .map(|(name, f)| (name.clone(), f.confidence))
            .collect()
    }

    pub fn finalize(&self) -> Result<(), OutputError> {
        let mut output = self.output.write().unwrap();
        output.compute_statistics();
        output.statistics.scan_duration_ms = self.start_time.elapsed().as_millis() as u64;
        drop(output);

        self.save()?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OutputSummary {
    pub total_functions: usize,
    pub total_structures: usize,
    pub total_classes: usize,
    pub total_properties: usize,
    pub total_methods: usize,
    pub total_constants: usize,
    pub total_offsets: usize,
    pub elapsed_ms: u64,
}

impl OutputSummary {
    pub fn display(&self) -> String {
        format!(
            "Functions: {}, Structures: {}, Classes: {}, Properties: {}, Methods: {}, Constants: {}, Total: {} ({}ms)",
            self.total_functions,
            self.total_structures,
            self.total_classes,
            self.total_properties,
            self.total_methods,
            self.total_constants,
            self.total_offsets,
            self.elapsed_ms
        )
    }
}

#[derive(Debug, Clone)]
pub enum OutputError {
    SaveError(String),
    LoadError(String),
    BackupError(String),
    ReportError(String),
    ValidationError(String),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::SaveError(e) => write!(f, "Save error: {}", e),
            OutputError::LoadError(e) => write!(f, "Load error: {}", e),
            OutputError::BackupError(e) => write!(f, "Backup error: {}", e),
            OutputError::ReportError(e) => write!(f, "Report error: {}", e),
            OutputError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for OutputError {}

pub fn create_output_manager(target_name: &str, output_path: PathBuf) -> OutputManager {
    OutputManager::new(target_name, output_path)
}
