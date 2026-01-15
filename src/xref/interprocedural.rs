// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::xref::{CallGraph, GraphNode, GraphEdge, EdgeKind, NodeKind};
use crate::xref::dataflow::DataLocation;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Context sensitivity level for inter-procedural analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSensitivity {
    /// No context sensitivity (fastest, least precise)
    Insensitive,
    /// Call-site sensitive (1-CFA)
    CallSite(usize),
    /// Object-sensitive
    ObjectSensitive(usize),
    /// Full context (most precise, slowest)
    Full,
}

/// Represents a calling context
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallContext {
    /// Stack of call sites
    pub call_sites: Vec<Address>,
    /// Maximum depth
    max_depth: usize,
}

impl CallContext {
    pub fn new(max_depth: usize) -> Self {
        Self {
            call_sites: Vec::new(),
            max_depth,
        }
    }

    pub fn push(&mut self, call_site: Address) {
        if self.call_sites.len() < self.max_depth {
            self.call_sites.push(call_site);
        }
    }

    pub fn pop(&mut self) -> Option<Address> {
        self.call_sites.pop()
    }

    pub fn depth(&self) -> usize {
        self.call_sites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.call_sites.is_empty()
    }

    pub fn matches(&self, other: &CallContext) -> bool {
        if self.call_sites.len() != other.call_sites.len() {
            return false;
        }
        self.call_sites.iter().zip(other.call_sites.iter())
            .all(|(a, b)| a == b)
    }
}

impl fmt::Display for CallContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sites: Vec<String> = self.call_sites.iter()
            .map(|a| format!("{:x}", a.as_u64()))
            .collect();
        write!(f, "[{}]", sites.join(" <- "))
    }
}

/// Summary of a function's effects for inter-procedural analysis
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    /// Function address
    pub address: Address,
    /// Function name (if known)
    pub name: Option<String>,
    /// Parameters used by the function
    pub parameters_used: Vec<DataLocation>,
    /// Return value locations
    pub return_values: Vec<DataLocation>,
    /// Global variables read
    pub globals_read: HashSet<u64>,
    /// Global variables written
    pub globals_written: HashSet<u64>,
    /// Functions called
    pub callees: Vec<Address>,
    /// Side effects
    pub side_effects: Vec<SideEffect>,
    /// Whether the function may not return
    pub may_not_return: bool,
    /// Whether the function is pure (no side effects)
    pub is_pure: bool,
}

impl FunctionSummary {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            name: None,
            parameters_used: Vec::new(),
            return_values: Vec::new(),
            globals_read: HashSet::new(),
            globals_written: HashSet::new(),
            callees: Vec::new(),
            side_effects: Vec::new(),
            may_not_return: false,
            is_pure: true,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn add_parameter(&mut self, param: DataLocation) {
        if !self.parameters_used.contains(&param) {
            self.parameters_used.push(param);
        }
    }

    pub fn add_return_value(&mut self, ret: DataLocation) {
        if !self.return_values.contains(&ret) {
            self.return_values.push(ret);
        }
    }

    pub fn add_global_read(&mut self, addr: u64) {
        self.globals_read.insert(addr);
    }

    pub fn add_global_write(&mut self, addr: u64) {
        self.globals_written.insert(addr);
        self.is_pure = false;
    }

    pub fn add_callee(&mut self, callee: Address) {
        if !self.callees.contains(&callee) {
            self.callees.push(callee);
        }
    }

    pub fn add_side_effect(&mut self, effect: SideEffect) {
        self.side_effects.push(effect);
        self.is_pure = false;
    }

    pub fn is_leaf(&self) -> bool {
        self.callees.is_empty()
    }

    pub fn modifies_globals(&self) -> bool {
        !self.globals_written.is_empty()
    }
}

impl fmt::Display for FunctionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Function Summary @ {:016x}", self.address.as_u64())?;
        if let Some(ref name) = self.name {
            writeln!(f, "  Name: {}", name)?;
        }
        writeln!(f, "  Parameters: {:?}", self.parameters_used)?;
        writeln!(f, "  Returns: {:?}", self.return_values)?;
        writeln!(f, "  Globals read: {}", self.globals_read.len())?;
        writeln!(f, "  Globals written: {}", self.globals_written.len())?;
        writeln!(f, "  Callees: {}", self.callees.len())?;
        writeln!(f, "  Pure: {}", self.is_pure)?;
        Ok(())
    }
}

/// Represents a side effect of a function
#[derive(Debug, Clone)]
pub enum SideEffect {
    /// Writes to memory at address
    MemoryWrite(u64),
    /// Reads from memory at address
    MemoryRead(u64),
    /// Calls external function
    ExternalCall(String),
    /// System call
    SystemCall(u32),
    /// I/O operation
    IoOperation,
    /// Exception/longjmp
    NonLocalJump,
}

impl fmt::Display for SideEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SideEffect::MemoryWrite(addr) => write!(f, "write@{:x}", addr),
            SideEffect::MemoryRead(addr) => write!(f, "read@{:x}", addr),
            SideEffect::ExternalCall(name) => write!(f, "call:{}", name),
            SideEffect::SystemCall(num) => write!(f, "syscall:{}", num),
            SideEffect::IoOperation => write!(f, "io"),
            SideEffect::NonLocalJump => write!(f, "nonlocal"),
        }
    }
}

/// Inter-procedural analyzer
pub struct InterproceduralAnalyzer {
    reader: Arc<dyn MemoryReader>,
    call_graph: CallGraph,
    function_summaries: HashMap<u64, FunctionSummary>,
    sensitivity: ContextSensitivity,
    analyzed_functions: HashSet<u64>,
    worklist: VecDeque<Address>,
}

impl InterproceduralAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            call_graph: CallGraph::new(),
            function_summaries: HashMap::new(),
            sensitivity: ContextSensitivity::Insensitive,
            analyzed_functions: HashSet::new(),
            worklist: VecDeque::new(),
        }
    }

    pub fn with_sensitivity(mut self, sensitivity: ContextSensitivity) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    /// Analyze starting from a set of entry points
    pub fn analyze(&mut self, entry_points: &[Address]) -> Result<InterproceduralResult, MemoryError> {
        // Initialize worklist with entry points
        for entry in entry_points {
            self.worklist.push_back(*entry);
        }

        // Process worklist
        while let Some(func_addr) = self.worklist.pop_front() {
            if self.analyzed_functions.contains(&func_addr.as_u64()) {
                continue;
            }

            self.analyze_function(func_addr)?;
        }

        // Build call graph from summaries
        self.build_call_graph();

        Ok(InterproceduralResult {
            call_graph: self.call_graph.clone(),
            function_summaries: self.function_summaries.clone(),
            reachable_functions: self.analyzed_functions.len(),
        })
    }

    /// Analyze a single function
    fn analyze_function(&mut self, addr: Address) -> Result<(), MemoryError> {
        let mut summary = FunctionSummary::new(addr);

        // Analyze function body
        let mut current = addr;
        let max_instructions = 2000;

        for _ in 0..max_instructions {
            let bytes = self.reader.read_bytes(current, 4)?;
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            // Analyze instruction
            self.analyze_instruction(&mut summary, current, insn);

            // Check for return
            if self.is_return_instruction(insn) {
                break;
            }

            // Check for unconditional branch (might be tail call)
            if self.is_unconditional_branch(insn) {
                let target = self.decode_branch_target(current, insn);
                if let Some(target_addr) = target {
                    // Could be tail call - add as callee
                    summary.add_callee(Address::new(target_addr));
                }
                break;
            }

            current = current + 4;
        }

        // Add default return value (x0 on ARM64)
        summary.add_return_value(DataLocation::reg(0));

        // Queue callees for analysis
        for callee in &summary.callees {
            if !self.analyzed_functions.contains(&callee.as_u64()) {
                self.worklist.push_back(*callee);
            }
        }

        self.analyzed_functions.insert(addr.as_u64());
        self.function_summaries.insert(addr.as_u64(), summary);

        Ok(())
    }

    /// Analyze a single instruction for inter-procedural effects
    fn analyze_instruction(&mut self, summary: &mut FunctionSummary, addr: Address, insn: u32) {
        let op = insn >> 26;

        // BL - Branch with Link (call)
        if (insn & 0xFC000000) == 0x94000000 {
            let offset = ((insn & 0x03FFFFFF) as i32) << 6 >> 6;
            let target = (addr.as_u64() as i64 + (offset as i64 * 4)) as u64;
            summary.add_callee(Address::new(target));
        }

        // BLR - Branch with Link to Register
        if (insn & 0xFFFFFC1F) == 0xD63F0000 {
            // Indirect call - harder to resolve
            let rn = ((insn >> 5) & 0x1F) as u8;
            // Could try to resolve if we have value tracking
        }

        // LDR from global
        if (insn & 0xFF000000) == 0x58000000 {
            // LDR (literal) - loading from PC-relative address
            let offset = ((insn >> 5) & 0x7FFFF) << 2;
            let target = addr.as_u64() + offset as u64;
            summary.add_global_read(target);
        }

        // STR to global would need more context

        // Track parameter usage (x0-x7 on ARM64)
        self.track_parameter_usage(summary, insn);
    }

    /// Track which parameters (x0-x7) are used
    fn track_parameter_usage(&self, summary: &mut FunctionSummary, insn: u32) {
        // Extract source registers from various instruction formats
        let rn = ((insn >> 5) & 0x1F) as u8;
        let rm = ((insn >> 16) & 0x1F) as u8;
        let rt = (insn & 0x1F) as u8;

        // If using x0-x7 as source, it's likely a parameter
        for r in [rn, rm] {
            if r <= 7 {
                summary.add_parameter(DataLocation::reg(r));
            }
        }
    }

    fn is_return_instruction(&self, insn: u32) -> bool {
        // RET
        (insn & 0xFFFFFC1F) == 0xD65F0000
    }

    fn is_unconditional_branch(&self, insn: u32) -> bool {
        // B (unconditional)
        (insn & 0xFC000000) == 0x14000000
    }

    fn decode_branch_target(&self, addr: Address, insn: u32) -> Option<u64> {
        if (insn & 0xFC000000) == 0x14000000 {
            let offset = ((insn & 0x03FFFFFF) as i32) << 6 >> 6;
            let target = (addr.as_u64() as i64 + (offset as i64 * 4)) as u64;
            Some(target)
        } else {
            None
        }
    }

    /// Build call graph from function summaries
    fn build_call_graph(&mut self) {
        self.call_graph = CallGraph::new();

        for (addr, summary) in &self.function_summaries {
            let node = GraphNode::new(
                Address::new(*addr),
                summary.name.clone().unwrap_or_else(|| format!("func_{:x}", addr)),
                NodeKind::Function,
            );
            self.call_graph.add_node(node);
        }

        for (addr, summary) in &self.function_summaries {
            for callee in &summary.callees {
                let edge = GraphEdge::new(
                    Address::new(*addr),
                    *callee,
                    EdgeKind::Call,
                );
                self.call_graph.add_edge(edge);
            }
        }
    }

    /// Get the function summary for an address
    pub fn get_summary(&self, addr: Address) -> Option<&FunctionSummary> {
        self.function_summaries.get(&addr.as_u64())
    }

    /// Get all pure functions
    pub fn get_pure_functions(&self) -> Vec<Address> {
        self.function_summaries.iter()
            .filter(|(_, s)| s.is_pure)
            .map(|(a, _)| Address::new(*a))
            .collect()
    }

    /// Get all leaf functions (no callees)
    pub fn get_leaf_functions(&self) -> Vec<Address> {
        self.function_summaries.iter()
            .filter(|(_, s)| s.is_leaf())
            .map(|(a, _)| Address::new(*a))
            .collect()
    }

    /// Find functions that may modify a global
    pub fn find_global_modifiers(&self, global_addr: u64) -> Vec<Address> {
        self.function_summaries.iter()
            .filter(|(_, s)| s.globals_written.contains(&global_addr))
            .map(|(a, _)| Address::new(*a))
            .collect()
    }

    /// Compute transitive callees for a function
    pub fn get_transitive_callees(&self, func: Address) -> HashSet<Address> {
        let mut result = HashSet::new();
        let mut worklist = VecDeque::new();
        worklist.push_back(func);

        while let Some(current) = worklist.pop_front() {
            if result.contains(&current) {
                continue;
            }
            result.insert(current);

            if let Some(summary) = self.function_summaries.get(&current.as_u64()) {
                for callee in &summary.callees {
                    if !result.contains(callee) {
                        worklist.push_back(*callee);
                    }
                }
            }
        }

        result.remove(&func); // Don't include the function itself
        result
    }

    /// Find strongly connected components (recursive function groups)
    pub fn find_recursive_groups(&self) -> Vec<Vec<Address>> {
        // Tarjan's algorithm for SCCs
        let mut index_counter = 0;
        let mut stack = Vec::new();
        let mut lowlinks: HashMap<u64, usize> = HashMap::new();
        let mut indices: HashMap<u64, usize> = HashMap::new();
        let mut on_stack: HashSet<u64> = HashSet::new();
        let mut sccs = Vec::new();

        fn strongconnect(
            v: u64,
            summaries: &HashMap<u64, FunctionSummary>,
            index_counter: &mut usize,
            stack: &mut Vec<u64>,
            lowlinks: &mut HashMap<u64, usize>,
            indices: &mut HashMap<u64, usize>,
            on_stack: &mut HashSet<u64>,
            sccs: &mut Vec<Vec<Address>>,
        ) {
            indices.insert(v, *index_counter);
            lowlinks.insert(v, *index_counter);
            *index_counter += 1;
            stack.push(v);
            on_stack.insert(v);

            if let Some(summary) = summaries.get(&v) {
                for callee in &summary.callees {
                    let w = callee.as_u64();
                    if !indices.contains_key(&w) {
                        strongconnect(w, summaries, index_counter, stack, lowlinks, indices, on_stack, sccs);
                        let low_w = *lowlinks.get(&w).unwrap_or(&0);
                        let low_v = *lowlinks.get(&v).unwrap_or(&0);
                        lowlinks.insert(v, low_v.min(low_w));
                    } else if on_stack.contains(&w) {
                        let idx_w = *indices.get(&w).unwrap_or(&0);
                        let low_v = *lowlinks.get(&v).unwrap_or(&0);
                        lowlinks.insert(v, low_v.min(idx_w));
                    }
                }
            }

            if lowlinks.get(&v) == indices.get(&v) {
                let mut scc = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    on_stack.remove(&w);
                    scc.push(Address::new(w));
                    if w == v {
                        break;
                    }
                }
                if scc.len() > 1 {
                    sccs.push(scc);
                }
            }
        }

        for addr in self.function_summaries.keys() {
            if !indices.contains_key(addr) {
                strongconnect(
                    *addr,
                    &self.function_summaries,
                    &mut index_counter,
                    &mut stack,
                    &mut lowlinks,
                    &mut indices,
                    &mut on_stack,
                    &mut sccs,
                );
            }
        }

        sccs
    }
}

/// Result of inter-procedural analysis
#[derive(Debug, Clone)]
pub struct InterproceduralResult {
    pub call_graph: CallGraph,
    pub function_summaries: HashMap<u64, FunctionSummary>,
    pub reachable_functions: usize,
}

impl InterproceduralResult {
    pub fn get_summary(&self, addr: Address) -> Option<&FunctionSummary> {
        self.function_summaries.get(&addr.as_u64())
    }

    pub fn function_count(&self) -> usize {
        self.function_summaries.len()
    }

    pub fn edge_count(&self) -> usize {
        self.call_graph.edge_count()
    }

    pub fn statistics(&self) -> InterproceduralStats {
        let pure_count = self.function_summaries.values()
            .filter(|s| s.is_pure)
            .count();
        let leaf_count = self.function_summaries.values()
            .filter(|s| s.is_leaf())
            .count();
        let total_callees: usize = self.function_summaries.values()
            .map(|s| s.callees.len())
            .sum();

        InterproceduralStats {
            total_functions: self.function_summaries.len(),
            pure_functions: pure_count,
            leaf_functions: leaf_count,
            total_edges: self.call_graph.edge_count(),
            avg_callees: if self.function_summaries.is_empty() {
                0.0
            } else {
                total_callees as f64 / self.function_summaries.len() as f64
            },
        }
    }
}

/// Statistics from inter-procedural analysis
#[derive(Debug, Clone)]
pub struct InterproceduralStats {
    pub total_functions: usize,
    pub pure_functions: usize,
    pub leaf_functions: usize,
    pub total_edges: usize,
    pub avg_callees: f64,
}

impl fmt::Display for InterproceduralStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Inter-procedural Analysis Statistics:")?;
        writeln!(f, "  Total functions: {}", self.total_functions)?;
        writeln!(f, "  Pure functions: {}", self.pure_functions)?;
        writeln!(f, "  Leaf functions: {}", self.leaf_functions)?;
        writeln!(f, "  Call edges: {}", self.total_edges)?;
        writeln!(f, "  Avg callees per function: {:.2}", self.avg_callees)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_context() {
        let mut ctx = CallContext::new(3);
        assert!(ctx.is_empty());

        ctx.push(Address::new(0x1000));
        ctx.push(Address::new(0x2000));
        assert_eq!(ctx.depth(), 2);

        let popped = ctx.pop();
        assert_eq!(popped, Some(Address::new(0x2000)));
    }

    #[test]
    fn test_function_summary() {
        let mut summary = FunctionSummary::new(Address::new(0x1000));
        summary.add_parameter(DataLocation::reg(0));
        summary.add_callee(Address::new(0x2000));

        assert!(!summary.is_leaf());
        assert!(summary.is_pure);

        summary.add_global_write(0x3000);
        assert!(!summary.is_pure);
    }
}
