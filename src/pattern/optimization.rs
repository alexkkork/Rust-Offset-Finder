// Tue Jan 15 2026 - Alex

use crate::pattern::Pattern;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Optimizes patterns for faster matching
pub struct PatternOptimizer {
    /// Minimum pattern length to consider for optimization
    min_length: usize,
    /// Maximum number of wildcards before skipping optimization
    max_wildcards: usize,
    /// Whether to use Boyer-Moore preprocessing
    use_boyer_moore: bool,
    /// Whether to combine similar patterns
    combine_patterns: bool,
}

impl PatternOptimizer {
    pub fn new() -> Self {
        Self {
            min_length: 4,
            max_wildcards: 8,
            use_boyer_moore: true,
            combine_patterns: true,
        }
    }

    pub fn with_min_length(mut self, len: usize) -> Self {
        self.min_length = len;
        self
    }

    pub fn with_max_wildcards(mut self, max: usize) -> Self {
        self.max_wildcards = max;
        self
    }

    pub fn without_boyer_moore(mut self) -> Self {
        self.use_boyer_moore = false;
        self
    }

    /// Optimize a single pattern
    pub fn optimize(&self, pattern: &Pattern) -> OptimizedPattern {
        let mut opt = OptimizedPattern::new(pattern.clone());

        // Skip if too short
        if pattern.len() < self.min_length {
            return opt;
        }

        // Count wildcards
        let wildcard_count = self.count_wildcards(pattern);
        if wildcard_count > self.max_wildcards {
            opt.warnings.push("Too many wildcards for effective optimization".to_string());
            return opt;
        }

        // Find best starting position (longest non-wildcard prefix)
        opt.best_start_offset = self.find_best_start(pattern);

        // Calculate skip tables for Boyer-Moore if enabled
        if self.use_boyer_moore {
            opt.bad_char_table = self.build_bad_char_table(pattern);
            opt.good_suffix_table = self.build_good_suffix_table(pattern);
        }

        // Find anchor points (fixed bytes in the pattern)
        opt.anchors = self.find_anchors(pattern);

        // Calculate selectivity score
        opt.selectivity = self.calculate_selectivity(pattern);

        opt.is_optimized = true;
        opt
    }

    /// Optimize multiple patterns together
    pub fn optimize_set(&self, patterns: &[Pattern]) -> OptimizedPatternSet {
        let mut opt_patterns: Vec<OptimizedPattern> = patterns.iter()
            .map(|p| self.optimize(p))
            .collect();

        // Sort by selectivity (most selective first)
        opt_patterns.sort_by(|a, b| b.selectivity.partial_cmp(&a.selectivity).unwrap());

        let mut set = OptimizedPatternSet::new();

        if self.combine_patterns {
            // Build a prefix trie for common prefixes
            let prefix_tree = self.build_prefix_tree(patterns);
            set.prefix_tree = Some(prefix_tree);
        }

        // Find patterns that can share skip tables
        let groups = self.group_similar_patterns(&opt_patterns);
        set.groups = groups;

        set.patterns = opt_patterns;
        set.combined_bad_char = self.build_combined_bad_char_table(patterns);

        set
    }

    fn count_wildcards(&self, pattern: &Pattern) -> usize {
        pattern.mask().iter().filter(|&&m| !m).count()
    }

    fn find_best_start(&self, pattern: &Pattern) -> usize {
        let bytes = pattern.bytes();
        let mask = pattern.mask();

        let mut best_start = 0;
        let mut best_length = 0;
        let mut current_start = 0;
        let mut current_length = 0;

        for (i, _byte) in bytes.iter().enumerate() {
            let is_fixed = mask.get(i).copied().unwrap_or(true);

            if is_fixed {
                if current_length == 0 {
                    current_start = i;
                }
                current_length += 1;
            } else {
                if current_length > best_length {
                    best_start = current_start;
                    best_length = current_length;
                }
                current_length = 0;
            }
        }

        if current_length > best_length {
            best_start = current_start;
        }

        best_start
    }

    fn build_bad_char_table(&self, pattern: &Pattern) -> HashMap<u8, usize> {
        let mut table = HashMap::new();
        let bytes = pattern.bytes();
        let mask = pattern.mask();
        let len = bytes.len();

        // Default shift is pattern length
        for byte in 0..=255u8 {
            table.insert(byte, len);
        }

        // Calculate shifts for each byte in pattern
        for (i, &byte) in bytes.iter().enumerate() {
            let is_fixed = mask.get(i).copied().unwrap_or(true);
            if is_fixed {
                table.insert(byte, len - 1 - i);
            }
        }

        table
    }

    fn build_good_suffix_table(&self, pattern: &Pattern) -> Vec<usize> {
        let bytes = pattern.bytes();
        let len = bytes.len();
        let mut table = vec![len; len];

        // Simplified good suffix calculation
        for i in 0..len {
            table[i] = len - i;
        }

        table
    }

    fn find_anchors(&self, pattern: &Pattern) -> Vec<(usize, u8)> {
        let mut anchors = Vec::new();
        let bytes = pattern.bytes();
        let mask = pattern.mask();

        // Find unique fixed bytes that are good anchors
        let mut byte_positions: HashMap<u8, Vec<usize>> = HashMap::new();

        for (i, &byte) in bytes.iter().enumerate() {
            let is_fixed = mask.get(i).copied().unwrap_or(true);
            if is_fixed {
                byte_positions.entry(byte).or_default().push(i);
            }
        }

        // Prefer bytes that appear only once (more selective)
        for (byte, positions) in byte_positions {
            if positions.len() == 1 {
                anchors.push((positions[0], byte));
            }
        }

        // Sort by position
        anchors.sort_by_key(|(pos, _)| *pos);
        anchors
    }

    fn calculate_selectivity(&self, pattern: &Pattern) -> f64 {
        let bytes = pattern.bytes();
        let mask = pattern.mask();
        let len = bytes.len() as f64;

        if len == 0.0 {
            return 0.0;
        }

        // Count fixed bytes
        let fixed_count = mask.iter().filter(|&&m| m).count();

        // Calculate byte entropy
        let mut byte_counts: HashMap<u8, usize> = HashMap::new();
        for (i, &byte) in bytes.iter().enumerate() {
            let is_fixed = mask.get(i).copied().unwrap_or(true);
            if is_fixed {
                *byte_counts.entry(byte).or_default() += 1;
            }
        }

        let unique_bytes = byte_counts.len() as f64;
        let fixed_ratio = fixed_count as f64 / len;
        let diversity = unique_bytes / len.min(256.0);

        // Selectivity score combines length, fixed ratio, and diversity
        (fixed_ratio * 0.4 + diversity * 0.3 + len.min(32.0) / 32.0 * 0.3).min(1.0)
    }

    fn build_prefix_tree(&self, patterns: &[Pattern]) -> PrefixTree {
        let mut tree = PrefixTree::new();

        for (i, pattern) in patterns.iter().enumerate() {
            tree.insert(pattern.bytes(), i);
        }

        tree
    }

    fn group_similar_patterns(&self, patterns: &[OptimizedPattern]) -> Vec<PatternGroup> {
        let mut groups: Vec<PatternGroup> = Vec::new();
        let mut used: HashSet<usize> = HashSet::new();

        for (i, pattern) in patterns.iter().enumerate() {
            if used.contains(&i) {
                continue;
            }

            let mut group = PatternGroup::new(i);
            used.insert(i);

            // Find similar patterns
            for (j, other) in patterns.iter().enumerate().skip(i + 1) {
                if !used.contains(&j) && self.patterns_similar(pattern, other) {
                    group.add_member(j);
                    used.insert(j);
                }
            }

            groups.push(group);
        }

        groups
    }

    fn patterns_similar(&self, a: &OptimizedPattern, b: &OptimizedPattern) -> bool {
        // Similar if they share bad char table entries
        let mut shared = 0;
        for (byte, shift_a) in &a.bad_char_table {
            if let Some(shift_b) = b.bad_char_table.get(byte) {
                if shift_a == shift_b {
                    shared += 1;
                }
            }
        }

        shared > 10 // Threshold for similarity
    }

    fn build_combined_bad_char_table(&self, patterns: &[Pattern]) -> HashMap<u8, usize> {
        let mut table: HashMap<u8, usize> = HashMap::new();
        
        // Use minimum shift for each byte across all patterns
        for pattern in patterns {
            let individual = self.build_bad_char_table(pattern);
            for (byte, shift) in individual {
                let entry = table.entry(byte).or_insert(usize::MAX);
                *entry = (*entry).min(shift);
            }
        }

        table
    }
}

impl Default for PatternOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// An optimized pattern with precomputed data
#[derive(Debug, Clone)]
pub struct OptimizedPattern {
    /// Original pattern
    pub pattern: Pattern,
    /// Best position to start matching
    pub best_start_offset: usize,
    /// Bad character table for Boyer-Moore
    pub bad_char_table: HashMap<u8, usize>,
    /// Good suffix table for Boyer-Moore
    pub good_suffix_table: Vec<usize>,
    /// Anchor points (fixed bytes with unique positions)
    pub anchors: Vec<(usize, u8)>,
    /// Selectivity score (0.0 - 1.0)
    pub selectivity: f64,
    /// Whether optimization was applied
    pub is_optimized: bool,
    /// Warnings during optimization
    pub warnings: Vec<String>,
}

impl OptimizedPattern {
    pub fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            best_start_offset: 0,
            bad_char_table: HashMap::new(),
            good_suffix_table: Vec::new(),
            anchors: Vec::new(),
            selectivity: 0.0,
            is_optimized: false,
            warnings: Vec::new(),
        }
    }

    /// Get the shift for a mismatched byte
    pub fn bad_char_shift(&self, byte: u8) -> usize {
        self.bad_char_table.get(&byte).copied().unwrap_or(self.pattern.len())
    }

    /// Check if a byte is an anchor
    pub fn is_anchor(&self, offset: usize, byte: u8) -> bool {
        self.anchors.iter().any(|(o, b)| *o == offset && *b == byte)
    }
}

impl fmt::Display for OptimizedPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OptimizedPattern:")?;
        writeln!(f, "  Length: {}", self.pattern.len())?;
        writeln!(f, "  Best start: {}", self.best_start_offset)?;
        writeln!(f, "  Anchors: {}", self.anchors.len())?;
        writeln!(f, "  Selectivity: {:.2}", self.selectivity)?;
        writeln!(f, "  Optimized: {}", self.is_optimized)?;
        for warn in &self.warnings {
            writeln!(f, "  Warning: {}", warn)?;
        }
        Ok(())
    }
}

/// A set of optimized patterns for multi-pattern matching
#[derive(Debug, Clone)]
pub struct OptimizedPatternSet {
    /// Individual optimized patterns
    pub patterns: Vec<OptimizedPattern>,
    /// Pattern groups for shared processing
    pub groups: Vec<PatternGroup>,
    /// Prefix tree for common prefix matching
    pub prefix_tree: Option<PrefixTree>,
    /// Combined bad character table
    pub combined_bad_char: HashMap<u8, usize>,
}

impl OptimizedPatternSet {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            groups: Vec::new(),
            prefix_tree: None,
            combined_bad_char: HashMap::new(),
        }
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Get combined shift for a byte
    pub fn combined_shift(&self, byte: u8) -> usize {
        self.combined_bad_char.get(&byte).copied().unwrap_or(1)
    }

    /// Get patterns ordered by selectivity
    pub fn patterns_by_selectivity(&self) -> Vec<(usize, &OptimizedPattern)> {
        let mut indexed: Vec<_> = self.patterns.iter().enumerate().collect();
        indexed.sort_by(|(_, a), (_, b)| b.selectivity.partial_cmp(&a.selectivity).unwrap());
        indexed
    }
}

impl Default for OptimizedPatternSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Group of similar patterns that can share processing
#[derive(Debug, Clone)]
pub struct PatternGroup {
    /// Index of the primary pattern
    pub primary: usize,
    /// Indices of other patterns in the group
    pub members: Vec<usize>,
}

impl PatternGroup {
    pub fn new(primary: usize) -> Self {
        Self {
            primary,
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, index: usize) {
        self.members.push(index);
    }

    pub fn size(&self) -> usize {
        1 + self.members.len()
    }

    pub fn all_indices(&self) -> Vec<usize> {
        let mut all = vec![self.primary];
        all.extend(&self.members);
        all
    }
}

/// Prefix tree for patterns
#[derive(Debug, Clone)]
pub struct PrefixTree {
    root: PrefixNode,
}

impl PrefixTree {
    pub fn new() -> Self {
        Self {
            root: PrefixNode::new(),
        }
    }

    pub fn insert(&mut self, bytes: &[u8], pattern_index: usize) {
        let mut node = &mut self.root;
        
        for &byte in bytes.iter().take(16) { // Only use first 16 bytes for prefix
            node = node.children.entry(byte).or_insert_with(PrefixNode::new);
        }
        
        node.pattern_indices.push(pattern_index);
    }

    pub fn find_matching_prefixes(&self, data: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();
        let mut node = &self.root;

        for &byte in data.iter().take(16) {
            if let Some(child) = node.children.get(&byte) {
                results.extend(&child.pattern_indices);
                node = child;
            } else {
                break;
            }
        }

        results
    }

    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    pub fn node_count(&self) -> usize {
        self.root.count_nodes()
    }
}

impl Default for PrefixTree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct PrefixNode {
    children: HashMap<u8, PrefixNode>,
    pattern_indices: Vec<usize>,
}

impl PrefixNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            pattern_indices: Vec::new(),
        }
    }

    fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.values().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    fn count_nodes(&self) -> usize {
        1 + self.children.values().map(|c| c.count_nodes()).sum::<usize>()
    }
}

/// Cache for pattern matching results
pub struct PatternCache {
    cache: HashMap<u64, CacheEntry>,
    max_size: usize,
    hits: usize,
    misses: usize,
}

impl PatternCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, pattern_hash: u64, region_hash: u64) -> Option<&Vec<u64>> {
        let key = pattern_hash ^ region_hash;
        if let Some(entry) = self.cache.get_mut(&key) {
            entry.last_used = std::time::Instant::now();
            self.hits += 1;
            Some(&entry.results)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, pattern_hash: u64, region_hash: u64, results: Vec<u64>) {
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }

        let key = pattern_hash ^ region_hash;
        self.cache.insert(key, CacheEntry {
            results,
            last_used: std::time::Instant::now(),
        });
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.cache.iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| *k)
        {
            self.cache.remove(&oldest_key);
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

struct CacheEntry {
    results: Vec<u64>,
    last_used: std::time::Instant,
}

/// Pattern generation from known functions
pub struct PatternGenerator;

impl PatternGenerator {
    /// Generate a pattern from function bytes
    pub fn from_function_bytes(bytes: &[u8], mask_registers: bool) -> Pattern {
        let mut pattern_bytes = bytes.to_vec();
        let mut mask = vec![0xFF; bytes.len()];

        if mask_registers && bytes.len() >= 4 {
            // Mask register fields in ARM64 instructions
            for i in (0..bytes.len()).step_by(4) {
                if i + 4 <= bytes.len() {
                    let insn = u32::from_le_bytes([bytes[i], bytes[i+1], bytes[i+2], bytes[i+3]]);
                    Self::mask_arm64_registers(insn, i, &mut pattern_bytes, &mut mask);
                }
            }
        }

        Pattern::with_mask(&pattern_bytes, &mask)
    }

    fn mask_arm64_registers(_insn: u32, offset: usize, _bytes: &mut [u8], mask: &mut [u8]) {
        // Mask the bottom 5 bits of first byte (rd) and bits 5-9 (rn) for most instructions
        // This is a simplified approach
        mask[offset] &= 0xE0; // Mask rd
        if mask.len() > offset + 1 {
            mask[offset + 1] &= 0xFC; // Partial mask for rn
        }
    }

    /// Generate patterns for function prologue
    pub fn function_prologue_patterns() -> Vec<Pattern> {
        vec![
            // STP x29, x30, [sp, #-N]!
            Pattern::with_mask(
                &[0xFD, 0x7B, 0xBF, 0xA9],
                &[0xFF, 0xFF, 0xC0, 0xFF]
            ),
            // SUB sp, sp, #N
            Pattern::with_mask(
                &[0xFF, 0x03, 0x00, 0xD1],
                &[0xFF, 0xFC, 0x00, 0xFF]
            ),
            // PACIBSP
            Pattern::from_bytes(&[0x7F, 0x23, 0x03, 0xD5]),
        ]
    }

    /// Generate patterns for function epilogue
    pub fn function_epilogue_patterns() -> Vec<Pattern> {
        vec![
            // LDP x29, x30, [sp], #N
            Pattern::with_mask(
                &[0xFD, 0x7B, 0xC1, 0xA8],
                &[0xFF, 0xFF, 0xC0, 0xFF]
            ),
            // RET
            Pattern::from_bytes(&[0xC0, 0x03, 0x5F, 0xD6]),
            // RETAB
            Pattern::from_bytes(&[0xFF, 0x0F, 0x5F, 0xD6]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_basic() {
        let optimizer = PatternOptimizer::new();
        let pattern = Pattern::from_bytes(&[0x48, 0x89, 0x5C, 0x24, 0x08]);
        let opt = optimizer.optimize(&pattern);

        assert!(opt.is_optimized);
        assert!(opt.selectivity > 0.0);
    }

    #[test]
    fn test_prefix_tree() {
        let mut tree = PrefixTree::new();
        tree.insert(&[0x48, 0x89, 0x5C], 0);
        tree.insert(&[0x48, 0x89, 0x4C], 1);
        tree.insert(&[0x48, 0x8B], 2);

        let matches = tree.find_matching_prefixes(&[0x48, 0x89, 0x5C, 0x24]);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_pattern_cache() {
        let mut cache = PatternCache::new(10);
        
        cache.insert(123, 456, vec![0x1000, 0x2000]);
        
        let result = cache.get(123, 456);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
        
        let miss = cache.get(999, 888);
        assert!(miss.is_none());
    }
}
