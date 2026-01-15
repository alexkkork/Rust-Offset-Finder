// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::structure::vtable::{VTable, VTableAnalyzer};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Represents a node in the class hierarchy tree
#[derive(Debug, Clone)]
pub struct ClassNode {
    /// Name of the class
    pub name: String,
    /// VTable address for this class
    pub vtable_address: Option<Address>,
    /// Size of the class instance in bytes
    pub instance_size: Option<usize>,
    /// Direct parent classes (supports multiple inheritance)
    pub parents: Vec<String>,
    /// Direct child classes
    pub children: Vec<String>,
    /// Depth in the hierarchy (0 = root)
    pub depth: usize,
    /// Whether this class is abstract (has pure virtuals)
    pub is_abstract: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ClassNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vtable_address: None,
            instance_size: None,
            parents: Vec::new(),
            children: Vec::new(),
            depth: 0,
            is_abstract: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_vtable(mut self, vtable: Address) -> Self {
        self.vtable_address = Some(vtable);
        self
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.instance_size = Some(size);
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parents.push(parent.to_string());
        self
    }

    pub fn add_child(&mut self, child: &str) {
        if !self.children.contains(&child.to_string()) {
            self.children.push(child.to_string());
        }
    }

    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn has_multiple_parents(&self) -> bool {
        self.parents.len() > 1
    }

    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl fmt::Display for ClassNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = "  ".repeat(self.depth);
        write!(f, "{}{}", indent, self.name)?;
        if let Some(vtable) = self.vtable_address {
            write!(f, " (vtable: {:016x})", vtable.as_u64())?;
        }
        if let Some(size) = self.instance_size {
            write!(f, " [size: {}]", size)?;
        }
        if self.is_abstract {
            write!(f, " [abstract]")?;
        }
        Ok(())
    }
}

/// Represents the complete class inheritance hierarchy
#[derive(Debug, Clone)]
pub struct ClassHierarchy {
    /// All classes in the hierarchy
    classes: HashMap<String, ClassNode>,
    /// Root classes (those with no parents)
    roots: Vec<String>,
    /// Leaf classes (those with no children)
    leaves: Vec<String>,
    /// Total depth of the hierarchy
    max_depth: usize,
}

impl ClassHierarchy {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            roots: Vec::new(),
            leaves: Vec::new(),
            max_depth: 0,
        }
    }

    /// Add a class to the hierarchy
    pub fn add_class(&mut self, node: ClassNode) {
        let name = node.name.clone();
        self.classes.insert(name.clone(), node);
        self.recalculate_relationships();
    }

    /// Add an inheritance relationship
    pub fn add_inheritance(&mut self, child: &str, parent: &str) {
        // Ensure both classes exist
        if !self.classes.contains_key(child) {
            self.classes.insert(child.to_string(), ClassNode::new(child));
        }
        if !self.classes.contains_key(parent) {
            self.classes.insert(parent.to_string(), ClassNode::new(parent));
        }

        // Add relationship
        if let Some(child_node) = self.classes.get_mut(child) {
            if !child_node.parents.contains(&parent.to_string()) {
                child_node.parents.push(parent.to_string());
            }
        }
        if let Some(parent_node) = self.classes.get_mut(parent) {
            if !parent_node.children.contains(&child.to_string()) {
                parent_node.children.push(child.to_string());
            }
        }

        self.recalculate_relationships();
    }

    /// Recalculate roots, leaves, and depths
    fn recalculate_relationships(&mut self) {
        // Find roots and leaves
        self.roots.clear();
        self.leaves.clear();

        for (name, node) in &self.classes {
            if node.parents.is_empty() {
                self.roots.push(name.clone());
            }
            if node.children.is_empty() {
                self.leaves.push(name.clone());
            }
        }

        // Calculate depths using BFS
        self.max_depth = 0;
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        for root in &self.roots {
            queue.push_back((root.clone(), 0));
        }

        while let Some((name, depth)) = queue.pop_front() {
            if visited.contains(&name) {
                continue;
            }
            visited.insert(name.clone());

            if let Some(node) = self.classes.get_mut(&name) {
                node.depth = depth;
                self.max_depth = self.max_depth.max(depth);

                for child in &node.children.clone() {
                    if !visited.contains(child) {
                        queue.push_back((child.clone(), depth + 1));
                    }
                }
            }
        }
    }

    /// Get a class by name
    pub fn get_class(&self, name: &str) -> Option<&ClassNode> {
        self.classes.get(name)
    }

    /// Get all ancestors of a class
    pub fn get_ancestors(&self, name: &str) -> Vec<&ClassNode> {
        let mut ancestors = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(node) = self.classes.get(name) {
            for parent in &node.parents {
                queue.push_back(parent.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(node) = self.classes.get(&current) {
                ancestors.push(node);
                for parent in &node.parents {
                    if !visited.contains(parent) {
                        queue.push_back(parent.clone());
                    }
                }
            }
        }

        ancestors
    }

    /// Get all descendants of a class
    pub fn get_descendants(&self, name: &str) -> Vec<&ClassNode> {
        let mut descendants = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(node) = self.classes.get(name) {
            for child in &node.children {
                queue.push_back(child.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(node) = self.classes.get(&current) {
                descendants.push(node);
                for child in &node.children {
                    if !visited.contains(child) {
                        queue.push_back(child.clone());
                    }
                }
            }
        }

        descendants
    }

    /// Check if one class is an ancestor of another
    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool {
        self.get_ancestors(descendant)
            .iter()
            .any(|node| node.name == ancestor)
    }

    /// Find the common ancestor of two classes
    pub fn find_common_ancestor(&self, class1: &str, class2: &str) -> Option<&ClassNode> {
        let ancestors1: HashSet<String> = self.get_ancestors(class1)
            .iter()
            .map(|n| n.name.clone())
            .collect();

        for ancestor in self.get_ancestors(class2) {
            if ancestors1.contains(&ancestor.name) {
                return Some(ancestor);
            }
        }

        None
    }

    /// Get classes at a specific depth
    pub fn get_classes_at_depth(&self, depth: usize) -> Vec<&ClassNode> {
        self.classes.values()
            .filter(|node| node.depth == depth)
            .collect()
    }

    /// Get all root classes
    pub fn get_roots(&self) -> Vec<&ClassNode> {
        self.roots.iter()
            .filter_map(|name| self.classes.get(name))
            .collect()
    }

    /// Get all leaf classes
    pub fn get_leaves(&self) -> Vec<&ClassNode> {
        self.leaves.iter()
            .filter_map(|name| self.classes.get(name))
            .collect()
    }

    /// Get the maximum depth of the hierarchy
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Get total number of classes
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Find classes using multiple inheritance
    pub fn find_multiple_inheritance(&self) -> Vec<&ClassNode> {
        self.classes.values()
            .filter(|node| node.has_multiple_parents())
            .collect()
    }

    /// Export hierarchy as DOT format for visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph ClassHierarchy {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n\n");

        for (name, node) in &self.classes {
            // Node definition
            let mut label = name.clone();
            if let Some(size) = node.instance_size {
                label.push_str(&format!("\\nsize: {}", size));
            }
            if node.is_abstract {
                dot.push_str(&format!("  \"{}\" [label=\"{}\", style=dashed];\n", name, label));
            } else {
                dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", name, label));
            }

            // Edges to children
            for child in &node.children {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", name, child));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Iterate over all classes
    pub fn iter(&self) -> impl Iterator<Item = &ClassNode> {
        self.classes.values()
    }

    /// Get mutable reference to a class
    pub fn get_class_mut(&mut self, name: &str) -> Option<&mut ClassNode> {
        self.classes.get_mut(name)
    }
}

impl Default for ClassHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ClassHierarchy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Class Hierarchy ({} classes, max depth {})", self.class_count(), self.max_depth)?;
        
        fn print_tree(f: &mut fmt::Formatter<'_>, hierarchy: &ClassHierarchy, name: &str, depth: usize) -> fmt::Result {
            let indent = "  ".repeat(depth);
            if let Some(node) = hierarchy.get_class(name) {
                writeln!(f, "{}{}", indent, node)?;
                for child in &node.children {
                    print_tree(f, hierarchy, child, depth + 1)?;
                }
            }
            Ok(())
        }

        for root in &self.roots {
            print_tree(f, self, root, 0)?;
        }

        Ok(())
    }
}

/// Detector for class inheritance relationships from binary analysis
pub struct InheritanceDetector {
    reader: Arc<dyn MemoryReader>,
    vtable_analyzer: VTableAnalyzer,
    hierarchy: ClassHierarchy,
}

impl InheritanceDetector {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            vtable_analyzer: VTableAnalyzer::new(reader),
            hierarchy: ClassHierarchy::new(),
        }
    }

    /// Detect inheritance relationships from vtables
    pub fn detect_from_vtables(&mut self, vtables: &[VTable]) -> &ClassHierarchy {
        // Add all classes
        for vtable in vtables {
            let node = ClassNode::new(&vtable.class_name)
                .with_vtable(vtable.address);
            self.hierarchy.add_class(node);
        }

        // Compare vtables to find inheritance
        for i in 0..vtables.len() {
            for j in 0..vtables.len() {
                if i == j {
                    continue;
                }

                let info = self.vtable_analyzer.detect_inheritance(&vtables[i], &vtables[j]);
                if info.is_likely_derived && info.confidence() > 0.6 {
                    self.hierarchy.add_inheritance(&info.child_class, &info.parent_class);
                }
            }
        }

        &self.hierarchy
    }

    /// Detect inheritance from instance layout
    pub fn detect_from_instance(&mut self, instance_addr: Address, class_name: &str) -> Result<Option<String>, MemoryError> {
        // Read vtable pointer (usually at offset 0)
        let vtable_ptr = self.reader.read_u64(instance_addr)?;
        
        if vtable_ptr == 0 || vtable_ptr < 0x100000000 {
            return Ok(None);
        }

        // Try to find parent by checking if vtable prefix matches known vtables
        for (name, node) in self.hierarchy.classes.iter() {
            if let Some(vt_addr) = node.vtable_address {
                if self.vtables_share_prefix(Address::new(vtable_ptr), vt_addr)? {
                    if name != class_name {
                        return Ok(Some(name.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if two vtables share a common prefix (suggesting inheritance)
    fn vtables_share_prefix(&self, vt1: Address, vt2: Address) -> Result<bool, MemoryError> {
        let mut matching = 0;
        for i in 0..8 {
            let entry1 = self.reader.read_u64(vt1 + (i * 8) as u64)?;
            let entry2 = self.reader.read_u64(vt2 + (i * 8) as u64)?;
            
            if entry1 == entry2 && entry1 != 0 {
                matching += 1;
            } else {
                break;
            }
        }

        Ok(matching >= 2)
    }

    /// Analyze RTTI to detect inheritance
    pub fn detect_from_rtti(&mut self, rtti_addr: Address) -> Result<Vec<String>, MemoryError> {
        let mut base_classes = Vec::new();

        // ARM64 RTTI structure (simplified):
        // +0x00: vtable for type_info
        // +0x08: name pointer
        // +0x10: base class pointer (for __si_class_type_info)
        
        // Try to read name
        let name_ptr = self.reader.read_u64(rtti_addr + 8)?;
        if name_ptr != 0 && name_ptr >= 0x100000000 {
            if let Ok(name) = self.reader.read_c_string(Address::new(name_ptr)) {
                if !name.is_empty() {
                    // Try to read base class
                    let base_ptr = self.reader.read_u64(rtti_addr + 16)?;
                    if base_ptr != 0 && base_ptr >= 0x100000000 {
                        let base_name_ptr = self.reader.read_u64(Address::new(base_ptr) + 8)?;
                        if base_name_ptr != 0 && base_name_ptr >= 0x100000000 {
                            if let Ok(base_name) = self.reader.read_c_string(Address::new(base_name_ptr)) {
                                if !base_name.is_empty() {
                                    base_classes.push(base_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(base_classes)
    }

    /// Get the detected hierarchy
    pub fn hierarchy(&self) -> &ClassHierarchy {
        &self.hierarchy
    }

    /// Get mutable hierarchy
    pub fn hierarchy_mut(&mut self) -> &mut ClassHierarchy {
        &mut self.hierarchy
    }
}

/// Statistics about the class hierarchy
#[derive(Debug, Clone)]
pub struct HierarchyStats {
    pub total_classes: usize,
    pub root_classes: usize,
    pub leaf_classes: usize,
    pub abstract_classes: usize,
    pub max_depth: usize,
    pub avg_depth: f64,
    pub multiple_inheritance_count: usize,
    pub single_inheritance_chains: usize,
}

impl HierarchyStats {
    pub fn from_hierarchy(hierarchy: &ClassHierarchy) -> Self {
        let total_classes = hierarchy.class_count();
        let root_classes = hierarchy.roots.len();
        let leaf_classes = hierarchy.leaves.len();
        let abstract_classes = hierarchy.classes.values()
            .filter(|n| n.is_abstract)
            .count();
        let max_depth = hierarchy.max_depth;
        let avg_depth = if total_classes > 0 {
            hierarchy.classes.values()
                .map(|n| n.depth as f64)
                .sum::<f64>() / total_classes as f64
        } else {
            0.0
        };
        let multiple_inheritance_count = hierarchy.find_multiple_inheritance().len();

        // Count single inheritance chains
        let single_inheritance_chains = hierarchy.classes.values()
            .filter(|n| n.parents.len() == 1 && n.children.len() == 1)
            .count();

        Self {
            total_classes,
            root_classes,
            leaf_classes,
            abstract_classes,
            max_depth,
            avg_depth,
            multiple_inheritance_count,
            single_inheritance_chains,
        }
    }
}

impl fmt::Display for HierarchyStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hierarchy Statistics:")?;
        writeln!(f, "  Total classes: {}", self.total_classes)?;
        writeln!(f, "  Root classes: {}", self.root_classes)?;
        writeln!(f, "  Leaf classes: {}", self.leaf_classes)?;
        writeln!(f, "  Abstract classes: {}", self.abstract_classes)?;
        writeln!(f, "  Max depth: {}", self.max_depth)?;
        writeln!(f, "  Average depth: {:.2}", self.avg_depth)?;
        writeln!(f, "  Multiple inheritance: {}", self.multiple_inheritance_count)?;
        writeln!(f, "  Single inheritance chains: {}", self.single_inheritance_chains)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_hierarchy() {
        let mut hierarchy = ClassHierarchy::new();

        hierarchy.add_class(ClassNode::new("Base"));
        hierarchy.add_class(ClassNode::new("Derived1").with_parent("Base"));
        hierarchy.add_class(ClassNode::new("Derived2").with_parent("Base"));
        hierarchy.add_class(ClassNode::new("GrandChild").with_parent("Derived1"));

        hierarchy.add_inheritance("Derived1", "Base");
        hierarchy.add_inheritance("Derived2", "Base");
        hierarchy.add_inheritance("GrandChild", "Derived1");

        assert_eq!(hierarchy.class_count(), 4);
        assert_eq!(hierarchy.max_depth(), 2);
        assert!(hierarchy.is_ancestor("Base", "GrandChild"));
    }

    #[test]
    fn test_multiple_inheritance() {
        let mut hierarchy = ClassHierarchy::new();

        hierarchy.add_class(ClassNode::new("Base1"));
        hierarchy.add_class(ClassNode::new("Base2"));
        hierarchy.add_class(ClassNode::new("Derived"));

        hierarchy.add_inheritance("Derived", "Base1");
        hierarchy.add_inheritance("Derived", "Base2");

        let mi_classes = hierarchy.find_multiple_inheritance();
        assert_eq!(mi_classes.len(), 1);
        assert_eq!(mi_classes[0].name, "Derived");
    }
}
