// Tue Jan 15 2026 - Alex

use crate::memory::Address;
use std::fmt;

/// Binary-level diff between two versions
#[derive(Debug, Clone)]
pub struct BinaryDiff {
    /// Old binary info
    pub old_version: String,
    /// New binary info
    pub new_version: String,
    /// List of changes
    pub changes: Vec<BinaryChange>,
    /// Regions that changed
    pub changed_regions: Vec<DiffRegion>,
    /// Statistics
    pub stats: DiffStats,
}

impl BinaryDiff {
    pub fn new(old_version: &str, new_version: &str) -> Self {
        Self {
            old_version: old_version.to_string(),
            new_version: new_version.to_string(),
            changes: Vec::new(),
            changed_regions: Vec::new(),
            stats: DiffStats::default(),
        }
    }

    pub fn add_change(&mut self, change: BinaryChange) {
        self.stats.update(&change);
        self.changes.push(change);
    }

    pub fn add_region(&mut self, region: DiffRegion) {
        self.changed_regions.push(region);
    }

    /// Compute diff from two byte arrays
    pub fn from_bytes(old_data: &[u8], new_data: &[u8], old_ver: &str, new_ver: &str) -> Self {
        let mut diff = Self::new(old_ver, new_ver);
        
        let min_len = old_data.len().min(new_data.len());
        let max_len = old_data.len().max(new_data.len());

        // Find byte-level differences
        let mut region_start: Option<usize> = None;
        
        for i in 0..min_len {
            if old_data[i] != new_data[i] {
                if region_start.is_none() {
                    region_start = Some(i);
                }
            } else if let Some(start) = region_start {
                // End of changed region
                diff.add_region(DiffRegion {
                    old_start: Address::new(start as u64),
                    new_start: Address::new(start as u64),
                    old_size: i - start,
                    new_size: i - start,
                    kind: RegionKind::Modified,
                });
                region_start = None;
            }
        }

        // Handle trailing change region
        if let Some(start) = region_start {
            diff.add_region(DiffRegion {
                old_start: Address::new(start as u64),
                new_start: Address::new(start as u64),
                old_size: min_len - start,
                new_size: min_len - start,
                kind: RegionKind::Modified,
            });
        }

        // Handle size difference
        if old_data.len() != new_data.len() {
            if new_data.len() > old_data.len() {
                diff.add_region(DiffRegion {
                    old_start: Address::new(old_data.len() as u64),
                    new_start: Address::new(old_data.len() as u64),
                    old_size: 0,
                    new_size: new_data.len() - old_data.len(),
                    kind: RegionKind::Added,
                });
            } else {
                diff.add_region(DiffRegion {
                    old_start: Address::new(new_data.len() as u64),
                    new_start: Address::new(new_data.len() as u64),
                    old_size: old_data.len() - new_data.len(),
                    new_size: 0,
                    kind: RegionKind::Removed,
                });
            }
        }

        diff.stats.old_size = old_data.len();
        diff.stats.new_size = new_data.len();
        diff.stats.changed_bytes = diff.changed_regions.iter()
            .map(|r| r.old_size.max(r.new_size))
            .sum();

        diff
    }

    /// Get changes in a specific address range
    pub fn changes_in_range(&self, start: Address, end: Address) -> Vec<&BinaryChange> {
        self.changes.iter()
            .filter(|c| c.old_address >= start && c.old_address < end)
            .collect()
    }

    /// Get percentage of bytes changed
    pub fn change_percentage(&self) -> f64 {
        let total = self.stats.old_size.max(self.stats.new_size);
        if total == 0 {
            0.0
        } else {
            (self.stats.changed_bytes as f64 / total as f64) * 100.0
        }
    }

    /// Check if a specific region is affected
    pub fn is_region_affected(&self, start: Address, size: usize) -> bool {
        let end = start + size as u64;
        self.changed_regions.iter().any(|r| {
            let region_end = r.old_start + r.old_size as u64;
            !(region_end <= start || r.old_start >= end)
        })
    }

    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    pub fn region_count(&self) -> usize {
        self.changed_regions.len()
    }
}

impl fmt::Display for BinaryDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Binary Diff: {} -> {}", self.old_version, self.new_version)?;
        writeln!(f, "  Changes: {}", self.changes.len())?;
        writeln!(f, "  Regions: {}", self.changed_regions.len())?;
        writeln!(f, "  Changed: {:.2}%", self.change_percentage())?;
        Ok(())
    }
}

/// A single binary change
#[derive(Debug, Clone)]
pub struct BinaryChange {
    /// Old address
    pub old_address: Address,
    /// New address (may differ due to insertions/deletions)
    pub new_address: Address,
    /// Kind of change
    pub kind: ChangeKind,
    /// Old bytes (empty for insertions)
    pub old_bytes: Vec<u8>,
    /// New bytes (empty for deletions)
    pub new_bytes: Vec<u8>,
    /// Optional description
    pub description: Option<String>,
}

impl BinaryChange {
    pub fn modification(old_addr: Address, new_addr: Address, old_bytes: Vec<u8>, new_bytes: Vec<u8>) -> Self {
        Self {
            old_address: old_addr,
            new_address: new_addr,
            kind: ChangeKind::Modified,
            old_bytes,
            new_bytes,
            description: None,
        }
    }

    pub fn insertion(addr: Address, bytes: Vec<u8>) -> Self {
        Self {
            old_address: addr,
            new_address: addr,
            kind: ChangeKind::Inserted,
            old_bytes: Vec::new(),
            new_bytes: bytes,
            description: None,
        }
    }

    pub fn deletion(addr: Address, bytes: Vec<u8>) -> Self {
        Self {
            old_address: addr,
            new_address: addr,
            kind: ChangeKind::Deleted,
            old_bytes: bytes,
            new_bytes: Vec::new(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn size(&self) -> usize {
        self.old_bytes.len().max(self.new_bytes.len())
    }

    pub fn delta(&self) -> i64 {
        self.new_bytes.len() as i64 - self.old_bytes.len() as i64
    }
}

impl fmt::Display for BinaryChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ 0x{:X}", self.kind, self.old_address.as_u64())?;
        if self.old_address != self.new_address {
            write!(f, " -> 0x{:X}", self.new_address.as_u64())?;
        }
        if let Some(ref desc) = self.description {
            write!(f, " ({})", desc)?;
        }
        Ok(())
    }
}

/// Kind of binary change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Bytes were modified
    Modified,
    /// Bytes were inserted
    Inserted,
    /// Bytes were deleted
    Deleted,
    /// Code was patched (semantically meaningful)
    Patched,
    /// Function was inlined
    Inlined,
    /// Function was outlined
    Outlined,
}

impl ChangeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeKind::Modified => "modified",
            ChangeKind::Inserted => "inserted",
            ChangeKind::Deleted => "deleted",
            ChangeKind::Patched => "patched",
            ChangeKind::Inlined => "inlined",
            ChangeKind::Outlined => "outlined",
        }
    }
}

/// A region that changed
#[derive(Debug, Clone)]
pub struct DiffRegion {
    /// Start address in old binary
    pub old_start: Address,
    /// Start address in new binary
    pub new_start: Address,
    /// Size in old binary
    pub old_size: usize,
    /// Size in new binary
    pub new_size: usize,
    /// Kind of region change
    pub kind: RegionKind,
}

impl DiffRegion {
    pub fn modified(start: Address, size: usize) -> Self {
        Self {
            old_start: start,
            new_start: start,
            old_size: size,
            new_size: size,
            kind: RegionKind::Modified,
        }
    }

    pub fn added(start: Address, size: usize) -> Self {
        Self {
            old_start: start,
            new_start: start,
            old_size: 0,
            new_size: size,
            kind: RegionKind::Added,
        }
    }

    pub fn removed(start: Address, size: usize) -> Self {
        Self {
            old_start: start,
            new_start: start,
            old_size: size,
            new_size: 0,
            kind: RegionKind::Removed,
        }
    }

    pub fn contains(&self, addr: Address) -> bool {
        let end = self.old_start + self.old_size as u64;
        addr >= self.old_start && addr < end
    }

    pub fn overlaps(&self, other: &DiffRegion) -> bool {
        let self_end = self.old_start + self.old_size as u64;
        let other_end = other.old_start + other.old_size as u64;
        !(self_end <= other.old_start || self.old_start >= other_end)
    }
}

impl fmt::Display for DiffRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: 0x{:X} ({} bytes) -> 0x{:X} ({} bytes)",
            self.kind,
            self.old_start.as_u64(), self.old_size,
            self.new_start.as_u64(), self.new_size
        )
    }
}

/// Kind of region change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    /// Region was modified
    Modified,
    /// Region was added
    Added,
    /// Region was removed
    Removed,
    /// Region was moved
    Moved,
    /// Region was resized
    Resized,
}

/// Diff statistics
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub old_size: usize,
    pub new_size: usize,
    pub changed_bytes: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub functions_changed: usize,
    pub strings_changed: usize,
}

impl DiffStats {
    pub fn update(&mut self, change: &BinaryChange) {
        match change.kind {
            ChangeKind::Inserted => {
                self.insertions += 1;
                self.changed_bytes += change.new_bytes.len();
            }
            ChangeKind::Deleted => {
                self.deletions += 1;
                self.changed_bytes += change.old_bytes.len();
            }
            ChangeKind::Modified | ChangeKind::Patched => {
                self.modifications += 1;
                self.changed_bytes += change.size();
            }
            _ => {}
        }
    }

    pub fn size_delta(&self) -> i64 {
        self.new_size as i64 - self.old_size as i64
    }

    pub fn total_changes(&self) -> usize {
        self.insertions + self.deletions + self.modifications
    }
}

impl fmt::Display for DiffStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Diff Statistics:")?;
        writeln!(f, "  Old size: {} bytes", self.old_size)?;
        writeln!(f, "  New size: {} bytes", self.new_size)?;
        writeln!(f, "  Size delta: {:+} bytes", self.size_delta())?;
        writeln!(f, "  Changed bytes: {}", self.changed_bytes)?;
        writeln!(f, "  Insertions: {}", self.insertions)?;
        writeln!(f, "  Deletions: {}", self.deletions)?;
        writeln!(f, "  Modifications: {}", self.modifications)?;
        Ok(())
    }
}

/// Function-level diff
#[derive(Debug, Clone)]
pub struct FunctionDiff {
    pub name: String,
    pub old_address: Option<Address>,
    pub new_address: Option<Address>,
    pub old_size: Option<usize>,
    pub new_size: Option<usize>,
    pub status: FunctionStatus,
    pub changes: Vec<InstructionChange>,
}

impl FunctionDiff {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            old_address: None,
            new_address: None,
            old_size: None,
            new_size: None,
            status: FunctionStatus::Unknown,
            changes: Vec::new(),
        }
    }

    pub fn unchanged(name: &str, addr: Address, size: usize) -> Self {
        Self {
            name: name.to_string(),
            old_address: Some(addr),
            new_address: Some(addr),
            old_size: Some(size),
            new_size: Some(size),
            status: FunctionStatus::Unchanged,
            changes: Vec::new(),
        }
    }

    pub fn was_moved(&self) -> bool {
        self.old_address != self.new_address && 
        self.old_address.is_some() && self.new_address.is_some()
    }

    pub fn size_changed(&self) -> bool {
        self.old_size != self.new_size
    }

    pub fn address_delta(&self) -> Option<i64> {
        match (self.old_address, self.new_address) {
            (Some(old), Some(new)) => Some(new.as_u64() as i64 - old.as_u64() as i64),
            _ => None,
        }
    }
}

/// Function status in diff
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionStatus {
    Unchanged,
    Modified,
    Added,
    Removed,
    Moved,
    Renamed,
    Unknown,
}

/// Instruction-level change
#[derive(Debug, Clone)]
pub struct InstructionChange {
    pub offset: usize,
    pub old_instruction: Option<String>,
    pub new_instruction: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_diff_from_bytes() {
        let old = vec![0x00, 0x01, 0x02, 0x03];
        let new = vec![0x00, 0xFF, 0x02, 0x03];
        
        let diff = BinaryDiff::from_bytes(&old, &new, "v1", "v2");
        assert!(!diff.changed_regions.is_empty());
    }

    #[test]
    fn test_binary_change() {
        let change = BinaryChange::modification(
            Address::new(0x1000),
            Address::new(0x1000),
            vec![0x90],
            vec![0xCC],
        );
        
        assert_eq!(change.size(), 1);
        assert_eq!(change.delta(), 0);
    }

    #[test]
    fn test_diff_region_contains() {
        let region = DiffRegion::modified(Address::new(0x1000), 0x100);
        assert!(region.contains(Address::new(0x1050)));
        assert!(!region.contains(Address::new(0x1100)));
    }
}
