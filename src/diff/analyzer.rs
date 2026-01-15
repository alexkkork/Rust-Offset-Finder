// Tue Jan 15 2026 - Alex

use crate::diff::binary::BinaryDiff;
use crate::diff::offset::{OffsetDiff, OffsetMigration};
use crate::diff::report::{DiffReport, DiffReportBuilder};
use crate::finders::result::FinderResult;
use std::collections::HashMap;
use std::fmt;

/// Main diff analyzer
pub struct DiffAnalyzer {
    old_version: String,
    new_version: String,
    old_results: HashMap<String, FinderResult>,
    new_results: HashMap<String, FinderResult>,
    binary_diff: Option<BinaryDiff>,
    offset_diff: Option<OffsetDiff>,
}

impl DiffAnalyzer {
    pub fn new(old_version: &str, new_version: &str) -> Self {
        Self {
            old_version: old_version.to_string(),
            new_version: new_version.to_string(),
            old_results: HashMap::new(),
            new_results: HashMap::new(),
            binary_diff: None,
            offset_diff: None,
        }
    }

    /// Set old version results
    pub fn with_old_results(mut self, results: HashMap<String, FinderResult>) -> Self {
        self.old_results = results;
        self
    }

    /// Set new version results
    pub fn with_new_results(mut self, results: HashMap<String, FinderResult>) -> Self {
        self.new_results = results;
        self
    }

    /// Analyze offset differences
    pub fn analyze_offsets(&mut self) -> &OffsetDiff {
        let diff = OffsetDiff::from_results(
            &self.old_results,
            &self.new_results,
            &self.old_version,
            &self.new_version,
        );
        self.offset_diff = Some(diff);
        self.offset_diff.as_ref().unwrap()
    }

    /// Analyze binary differences
    pub fn analyze_binary(&mut self, old_data: &[u8], new_data: &[u8]) -> &BinaryDiff {
        let diff = BinaryDiff::from_bytes(old_data, new_data, &self.old_version, &self.new_version);
        self.binary_diff = Some(diff);
        self.binary_diff.as_ref().unwrap()
    }

    /// Get full analysis result
    pub fn analyze(&mut self) -> DiffResult {
        // Ensure offset diff is computed
        if self.offset_diff.is_none() && (!self.old_results.is_empty() || !self.new_results.is_empty()) {
            self.analyze_offsets();
        }

        let summary = self.compute_summary();

        DiffResult {
            old_version: self.old_version.clone(),
            new_version: self.new_version.clone(),
            offset_diff: self.offset_diff.clone(),
            binary_diff: self.binary_diff.clone(),
            summary,
        }
    }

    fn compute_summary(&self) -> DiffSummary {
        let mut summary = DiffSummary::default();

        if let Some(ref diff) = self.offset_diff {
            summary.offset_changes = diff.changes.len();
            summary.offsets_unchanged = diff.unchanged.len();
            summary.breaking_changes = diff.changes.iter()
                .filter(|c| c.is_breaking())
                .count();
        }

        if let Some(ref diff) = self.binary_diff {
            summary.binary_regions_changed = diff.changed_regions.len();
            summary.bytes_changed = diff.stats.changed_bytes;
        }

        summary
    }

    /// Generate migration from analysis
    pub fn generate_migration(&self) -> Option<OffsetMigration> {
        self.offset_diff.as_ref().map(OffsetMigration::from_diff)
    }

    /// Generate report from analysis
    pub fn generate_report(&self) -> DiffReport {
        let mut builder = DiffReportBuilder::new(&self.old_version, &self.new_version);

        if let Some(ref diff) = self.offset_diff {
            builder = builder.offset_diff(diff.clone());
        }

        if let Some(ref diff) = self.binary_diff {
            builder = builder.binary_diff(diff.clone());
        }

        builder.build()
    }
}

/// Result of diff analysis
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub old_version: String,
    pub new_version: String,
    pub offset_diff: Option<OffsetDiff>,
    pub binary_diff: Option<BinaryDiff>,
    pub summary: DiffSummary,
}

impl DiffResult {
    /// Check if any changes were found
    pub fn has_changes(&self) -> bool {
        self.summary.total_changes() > 0
    }

    /// Check if there are breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.summary.breaking_changes > 0
    }

    /// Get migration info
    pub fn get_migration(&self) -> Option<OffsetMigration> {
        self.offset_diff.as_ref().map(OffsetMigration::from_diff)
    }

    /// Get list of changed offset names
    pub fn changed_offsets(&self) -> Vec<&str> {
        self.offset_diff.as_ref()
            .map(|d| d.changes.iter().map(|c| c.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get list of unchanged offset names
    pub fn unchanged_offsets(&self) -> Vec<&str> {
        self.offset_diff.as_ref()
            .map(|d| d.unchanged.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

impl fmt::Display for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Diff Result: {} -> {}", self.old_version, self.new_version)?;
        writeln!(f, "{}", self.summary)?;
        
        if let Some(ref diff) = self.offset_diff {
            writeln!(f, "\nOffset Changes:")?;
            for change in &diff.changes {
                writeln!(f, "  {}", change)?;
            }
        }
        
        Ok(())
    }
}

/// Summary of diff analysis
#[derive(Debug, Clone, Default)]
pub struct DiffSummary {
    pub offset_changes: usize,
    pub offsets_unchanged: usize,
    pub breaking_changes: usize,
    pub binary_regions_changed: usize,
    pub bytes_changed: usize,
    pub functions_changed: usize,
    pub strings_changed: usize,
}

impl DiffSummary {
    pub fn total_changes(&self) -> usize {
        self.offset_changes + self.binary_regions_changed
    }

    pub fn stability_score(&self) -> f64 {
        let total = self.offset_changes + self.offsets_unchanged;
        if total == 0 {
            1.0
        } else {
            self.offsets_unchanged as f64 / total as f64
        }
    }
}

impl fmt::Display for DiffSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Summary:")?;
        writeln!(f, "  Offset changes: {}", self.offset_changes)?;
        writeln!(f, "  Unchanged: {}", self.offsets_unchanged)?;
        writeln!(f, "  Breaking: {}", self.breaking_changes)?;
        writeln!(f, "  Stability: {:.1}%", self.stability_score() * 100.0)?;
        Ok(())
    }
}

/// Batch diff analyzer for multiple versions
pub struct BatchDiffAnalyzer {
    versions: Vec<VersionData>,
}

impl BatchDiffAnalyzer {
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
        }
    }

    pub fn add_version(&mut self, version: &str, results: HashMap<String, FinderResult>) {
        self.versions.push(VersionData {
            version: version.to_string(),
            results,
        });
    }

    /// Analyze all consecutive version pairs
    pub fn analyze_all(&self) -> Vec<DiffResult> {
        let mut results = Vec::new();

        for i in 0..self.versions.len().saturating_sub(1) {
            let old = &self.versions[i];
            let new = &self.versions[i + 1];

            let mut analyzer = DiffAnalyzer::new(&old.version, &new.version)
                .with_old_results(old.results.clone())
                .with_new_results(new.results.clone());

            results.push(analyzer.analyze());
        }

        results
    }

    /// Get offset stability across all versions
    pub fn offset_stability(&self, name: &str) -> OffsetStability {
        let mut values: Vec<(String, Option<u64>)> = Vec::new();

        for v in &self.versions {
            let offset = v.results.get(name).map(|r| r.address.as_u64());
            values.push((v.version.clone(), offset));
        }

        let changes = values.windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .count();

        OffsetStability {
            name: name.to_string(),
            versions_present: values.iter().filter(|(_, o)| o.is_some()).count(),
            total_versions: values.len(),
            changes,
            values,
        }
    }

    /// Get all offset stabilities
    pub fn all_stabilities(&self) -> Vec<OffsetStability> {
        let mut names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        
        for v in &self.versions {
            for name in v.results.keys() {
                names.insert(name);
            }
        }

        names.iter()
            .map(|name| self.offset_stability(name))
            .collect()
    }
}

impl Default for BatchDiffAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

struct VersionData {
    version: String,
    results: HashMap<String, FinderResult>,
}

/// Stability information for an offset
#[derive(Debug, Clone)]
pub struct OffsetStability {
    pub name: String,
    pub versions_present: usize,
    pub total_versions: usize,
    pub changes: usize,
    pub values: Vec<(String, Option<u64>)>,
}

impl OffsetStability {
    pub fn stability_score(&self) -> f64 {
        if self.total_versions <= 1 {
            1.0
        } else {
            1.0 - (self.changes as f64 / (self.total_versions - 1) as f64)
        }
    }

    pub fn is_stable(&self) -> bool {
        self.changes == 0
    }

    pub fn was_removed(&self) -> bool {
        self.values.last().map(|(_, o)| o.is_none()).unwrap_or(false)
    }

    pub fn was_added_in(&self) -> Option<&str> {
        for (i, (_, offset)) in self.values.iter().enumerate() {
            if offset.is_some() {
                if i == 0 || self.values[i-1].1.is_none() {
                    return Some(&self.values[i].0);
                }
            }
        }
        None
    }
}

impl fmt::Display for OffsetStability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:.0}% stable ({} changes across {} versions)",
            self.name,
            self.stability_score() * 100.0,
            self.changes,
            self.total_versions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_analyzer() {
        let mut old_results = HashMap::new();
        old_results.insert("test".to_string(), FinderResult::new("test".to_string(), Address::new(0x1000), 0.9));

        let mut new_results = HashMap::new();
        new_results.insert("test".to_string(), FinderResult::new("test".to_string(), Address::new(0x1100), 0.9));

        let mut analyzer = DiffAnalyzer::new("v1", "v2")
            .with_old_results(old_results)
            .with_new_results(new_results);

        let result = analyzer.analyze();
        assert!(result.has_changes());
        assert_eq!(result.summary.offset_changes, 1);
    }

    #[test]
    fn test_offset_stability() {
        let mut batch = BatchDiffAnalyzer::new();
        
        let mut v1 = HashMap::new();
        v1.insert("offset_a".to_string(), FinderResult::new("offset_a".to_string(), Address::new(0x1000), 0.9));
        batch.add_version("v1", v1);

        let mut v2 = HashMap::new();
        v2.insert("offset_a".to_string(), FinderResult::new("offset_a".to_string(), Address::new(0x1000), 0.9));
        batch.add_version("v2", v2);

        let stability = batch.offset_stability("offset_a");
        assert!(stability.is_stable());
        assert_eq!(stability.stability_score(), 1.0);
    }
}
