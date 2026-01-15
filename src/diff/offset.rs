// Tue Jan 15 2026 - Alex

use crate::finders::result::FinderResult;
use std::collections::HashMap;
use std::fmt;

/// Diff between offset values across versions
#[derive(Debug, Clone)]
pub struct OffsetDiff {
    pub old_version: String,
    pub new_version: String,
    pub changes: Vec<OffsetChange>,
    pub unchanged: Vec<String>,
    pub migration: Option<OffsetMigration>,
}

impl OffsetDiff {
    pub fn new(old_version: &str, new_version: &str) -> Self {
        Self {
            old_version: old_version.to_string(),
            new_version: new_version.to_string(),
            changes: Vec::new(),
            unchanged: Vec::new(),
            migration: None,
        }
    }

    /// Compute diff from two FinderResult sets
    pub fn from_results(
        old_results: &HashMap<String, FinderResult>,
        new_results: &HashMap<String, FinderResult>,
        old_ver: &str,
        new_ver: &str,
    ) -> Self {
        let mut diff = Self::new(old_ver, new_ver);

        // Find changed and unchanged offsets
        for (name, old_result) in old_results {
            let old_addr = old_result.address.as_u64();
            if let Some(new_result) = new_results.get(name) {
                let new_addr = new_result.address.as_u64();
                if old_addr != new_addr {
                    diff.changes.push(OffsetChange {
                        name: name.clone(),
                        old_value: Some(old_addr),
                        new_value: Some(new_addr),
                        old_confidence: old_result.confidence,
                        new_confidence: new_result.confidence,
                        kind: OffsetChangeKind::ValueChanged,
                        delta: new_addr as i64 - old_addr as i64,
                    });
                } else {
                    diff.unchanged.push(name.clone());
                }
            } else {
                diff.changes.push(OffsetChange {
                    name: name.clone(),
                    old_value: Some(old_addr),
                    new_value: None,
                    old_confidence: old_result.confidence,
                    new_confidence: 0.0,
                    kind: OffsetChangeKind::Removed,
                    delta: 0,
                });
            }
        }

        // Find new offsets
        for (name, new_result) in new_results {
            if !old_results.contains_key(name) {
                diff.changes.push(OffsetChange {
                    name: name.clone(),
                    old_value: None,
                    new_value: Some(new_result.address.as_u64()),
                    old_confidence: 0.0,
                    new_confidence: new_result.confidence,
                    kind: OffsetChangeKind::Added,
                    delta: 0,
                });
            }
        }

        // Sort changes by name
        diff.changes.sort_by(|a, b| a.name.cmp(&b.name));

        diff
    }

    /// Get changes of a specific kind
    pub fn changes_of_kind(&self, kind: OffsetChangeKind) -> Vec<&OffsetChange> {
        self.changes.iter().filter(|c| c.kind == kind).collect()
    }

    /// Generate migration info
    pub fn generate_migration(&mut self) {
        let migration = OffsetMigration::from_diff(self);
        self.migration = Some(migration);
    }

    /// Get overall change statistics
    pub fn statistics(&self) -> OffsetDiffStats {
        let mut stats = OffsetDiffStats::default();
        
        stats.total = self.changes.len() + self.unchanged.len();
        stats.unchanged = self.unchanged.len();
        
        for change in &self.changes {
            match change.kind {
                OffsetChangeKind::ValueChanged => stats.changed += 1,
                OffsetChangeKind::Added => stats.added += 1,
                OffsetChangeKind::Removed => stats.removed += 1,
                OffsetChangeKind::TypeChanged => stats.type_changed += 1,
            }
        }

        stats
    }

    /// Check if any breaking changes exist
    pub fn has_breaking_changes(&self) -> bool {
        self.changes.iter().any(|c| c.is_breaking())
    }

    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    pub fn unchanged_count(&self) -> usize {
        self.unchanged.len()
    }
}

impl fmt::Display for OffsetDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Offset Diff: {} -> {}", self.old_version, self.new_version)?;
        
        let stats = self.statistics();
        writeln!(f, "  Total: {}, Unchanged: {}, Changed: {}, Added: {}, Removed: {}",
            stats.total, stats.unchanged, stats.changed, stats.added, stats.removed)?;
        
        if !self.changes.is_empty() {
            writeln!(f, "\nChanges:")?;
            for change in &self.changes {
                writeln!(f, "  {}", change)?;
            }
        }
        
        Ok(())
    }
}

/// A single offset change
#[derive(Debug, Clone)]
pub struct OffsetChange {
    pub name: String,
    pub old_value: Option<u64>,
    pub new_value: Option<u64>,
    pub old_confidence: f64,
    pub new_confidence: f64,
    pub kind: OffsetChangeKind,
    pub delta: i64,
}

impl OffsetChange {
    pub fn is_breaking(&self) -> bool {
        matches!(self.kind, OffsetChangeKind::ValueChanged | OffsetChangeKind::Removed)
    }

    pub fn confidence_delta(&self) -> f64 {
        self.new_confidence - self.old_confidence
    }

    pub fn old_hex(&self) -> String {
        self.old_value.map(|v| format!("0x{:X}", v)).unwrap_or_else(|| "-".to_string())
    }

    pub fn new_hex(&self) -> String {
        self.new_value.map(|v| format!("0x{:X}", v)).unwrap_or_else(|| "-".to_string())
    }
}

impl fmt::Display for OffsetChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            OffsetChangeKind::ValueChanged => {
                write!(f, "{}: {} -> {} (delta: {:+})", 
                    self.name, self.old_hex(), self.new_hex(), self.delta)
            }
            OffsetChangeKind::Added => {
                write!(f, "{}: [NEW] {}", self.name, self.new_hex())
            }
            OffsetChangeKind::Removed => {
                write!(f, "{}: {} [REMOVED]", self.name, self.old_hex())
            }
            OffsetChangeKind::TypeChanged => {
                write!(f, "{}: type changed", self.name)
            }
        }
    }
}

/// Kind of offset change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetChangeKind {
    ValueChanged,
    Added,
    Removed,
    TypeChanged,
}

/// Statistics for offset diff
#[derive(Debug, Clone, Default)]
pub struct OffsetDiffStats {
    pub total: usize,
    pub unchanged: usize,
    pub changed: usize,
    pub added: usize,
    pub removed: usize,
    pub type_changed: usize,
}

impl OffsetDiffStats {
    pub fn change_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            ((self.changed + self.added + self.removed) as f64 / self.total as f64) * 100.0
        }
    }
}

/// Migration information for updating offsets
#[derive(Debug, Clone)]
pub struct OffsetMigration {
    pub from_version: String,
    pub to_version: String,
    pub mappings: Vec<OffsetMapping>,
    pub strategy: MigrationStrategy,
    pub warnings: Vec<String>,
}

impl OffsetMigration {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from_version: from.to_string(),
            to_version: to.to_string(),
            mappings: Vec::new(),
            strategy: MigrationStrategy::Direct,
            warnings: Vec::new(),
        }
    }

    pub fn from_diff(diff: &OffsetDiff) -> Self {
        let mut migration = Self::new(&diff.old_version, &diff.new_version);

        for change in &diff.changes {
            match change.kind {
                OffsetChangeKind::ValueChanged => {
                    migration.mappings.push(OffsetMapping {
                        name: change.name.clone(),
                        old_offset: change.old_value,
                        new_offset: change.new_value,
                        transform: Some(OffsetTransform::Delta(change.delta)),
                    });
                }
                OffsetChangeKind::Removed => {
                    migration.warnings.push(format!("Offset '{}' was removed", change.name));
                    migration.mappings.push(OffsetMapping {
                        name: change.name.clone(),
                        old_offset: change.old_value,
                        new_offset: None,
                        transform: None,
                    });
                }
                OffsetChangeKind::Added => {
                    migration.mappings.push(OffsetMapping {
                        name: change.name.clone(),
                        old_offset: None,
                        new_offset: change.new_value,
                        transform: None,
                    });
                }
                _ => {}
            }
        }

        // Detect common delta pattern
        let deltas: Vec<i64> = diff.changes.iter()
            .filter(|c| c.kind == OffsetChangeKind::ValueChanged)
            .map(|c| c.delta)
            .collect();

        if !deltas.is_empty() {
            let first = deltas[0];
            if deltas.iter().all(|&d| d == first) {
                migration.strategy = MigrationStrategy::UniformDelta(first);
            } else {
                migration.strategy = MigrationStrategy::Mixed;
            }
        }

        migration
    }

    /// Apply migration to an offset value
    pub fn migrate(&self, name: &str, old_value: u64) -> Option<u64> {
        // Try specific mapping first
        for mapping in &self.mappings {
            if mapping.name == name {
                return mapping.new_offset;
            }
        }

        // Apply strategy-based migration
        match self.strategy {
            MigrationStrategy::UniformDelta(delta) => {
                Some((old_value as i64 + delta) as u64)
            }
            MigrationStrategy::Direct => Some(old_value),
            MigrationStrategy::Mixed => None,
        }
    }

    /// Generate migration code/script
    pub fn to_migration_script(&self) -> String {
        let mut script = String::new();
        
        script.push_str(&format!("// Migration: {} -> {}\n\n", 
            self.from_version, self.to_version));

        match self.strategy {
            MigrationStrategy::UniformDelta(delta) => {
                script.push_str(&format!("// All offsets shifted by {:+}\n", delta));
                script.push_str(&format!("const OFFSET_DELTA: i64 = {};\n\n", delta));
            }
            _ => {}
        }

        script.push_str("fn migrate_offset(name: &str, old: u64) -> Option<u64> {\n");
        script.push_str("    match name {\n");

        for mapping in &self.mappings {
            if let (Some(old), Some(new)) = (mapping.old_offset, mapping.new_offset) {
                script.push_str(&format!("        \"{}\" => Some(0x{:X}), // was 0x{:X}\n", 
                    mapping.name, new, old));
            } else if mapping.new_offset.is_none() {
                script.push_str(&format!("        \"{}\" => None, // REMOVED\n", mapping.name));
            }
        }

        script.push_str("        _ => None,\n");
        script.push_str("    }\n");
        script.push_str("}\n");

        script
    }

    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl fmt::Display for OffsetMigration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Migration: {} -> {}", self.from_version, self.to_version)?;
        writeln!(f, "Strategy: {:?}", self.strategy)?;
        writeln!(f, "Mappings: {}", self.mappings.len())?;
        
        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warn in &self.warnings {
                writeln!(f, "  - {}", warn)?;
            }
        }
        
        Ok(())
    }
}

/// Migration strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStrategy {
    /// All offsets are unchanged
    Direct,
    /// All offsets shifted by same delta
    UniformDelta(i64),
    /// Mixed changes, need per-offset mapping
    Mixed,
}

/// Mapping for a single offset
#[derive(Debug, Clone)]
pub struct OffsetMapping {
    pub name: String,
    pub old_offset: Option<u64>,
    pub new_offset: Option<u64>,
    pub transform: Option<OffsetTransform>,
}

impl OffsetMapping {
    pub fn is_removed(&self) -> bool {
        self.new_offset.is_none() && self.old_offset.is_some()
    }

    pub fn is_added(&self) -> bool {
        self.old_offset.is_none() && self.new_offset.is_some()
    }

    pub fn delta(&self) -> Option<i64> {
        match (self.old_offset, self.new_offset) {
            (Some(old), Some(new)) => Some(new as i64 - old as i64),
            _ => None,
        }
    }
}

/// Transform to apply to offset
#[derive(Debug, Clone)]
pub enum OffsetTransform {
    /// Add a constant delta
    Delta(i64),
    /// Multiply by factor
    Scale(f64),
    /// Custom expression
    Expression(String),
}

/// Multi-version offset history
#[derive(Debug, Clone)]
pub struct OffsetHistory {
    pub name: String,
    pub entries: Vec<OffsetHistoryEntry>,
}

impl OffsetHistory {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, version: &str, offset: Option<u64>, confidence: f64) {
        self.entries.push(OffsetHistoryEntry {
            version: version.to_string(),
            offset,
            confidence,
            notes: None,
        });
    }

    pub fn latest(&self) -> Option<&OffsetHistoryEntry> {
        self.entries.last()
    }

    pub fn for_version(&self, version: &str) -> Option<&OffsetHistoryEntry> {
        self.entries.iter().find(|e| e.version == version)
    }

    pub fn was_stable(&self) -> bool {
        if self.entries.len() < 2 {
            return true;
        }
        
        let offsets: Vec<_> = self.entries.iter()
            .filter_map(|e| e.offset)
            .collect();
        
        offsets.windows(2).all(|w| w[0] == w[1])
    }

    pub fn change_count(&self) -> usize {
        if self.entries.len() < 2 {
            return 0;
        }

        self.entries.windows(2)
            .filter(|w| w[0].offset != w[1].offset)
            .count()
    }
}

/// Single entry in offset history
#[derive(Debug, Clone)]
pub struct OffsetHistoryEntry {
    pub version: String,
    pub offset: Option<u64>,
    pub confidence: f64,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_change() {
        let change = OffsetChange {
            name: "test".to_string(),
            old_value: Some(0x1000),
            new_value: Some(0x1100),
            old_confidence: 0.9,
            new_confidence: 0.95,
            kind: OffsetChangeKind::ValueChanged,
            delta: 0x100,
        };

        assert!(change.is_breaking());
        assert_eq!(change.delta, 0x100);
    }

    #[test]
    fn test_offset_history() {
        let mut history = OffsetHistory::new("test_offset");
        history.add_entry("v1", Some(0x1000), 0.9);
        history.add_entry("v2", Some(0x1000), 0.95);
        history.add_entry("v3", Some(0x1100), 0.9);

        assert_eq!(history.change_count(), 1);
        assert!(!history.was_stable());
    }
}
