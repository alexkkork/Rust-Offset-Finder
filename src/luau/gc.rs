// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::types::{GCHeader, TypeTag};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

pub struct GcAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl GcAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze_gc_state(&self, global_state: Address) -> Result<GcStateInfo, MemoryError> {
        let mut info = GcStateInfo::new();

        Ok(info)
    }

    pub fn walk_gc_list(&self, head: Address, max_objects: usize) -> Result<Vec<GcObjectInfo>, MemoryError> {
        let mut objects = Vec::new();
        let mut visited = HashSet::new();
        let mut current = head;

        while current.as_u64() != 0 && objects.len() < max_objects {
            if !visited.insert(current.as_u64()) {
                break;
            }

            let header_data = self.reader.read_bytes(current, 16)?;
            if let Some(header) = GCHeader::from_bytes(&header_data) {
                objects.push(GcObjectInfo {
                    address: current,
                    type_tag: header.tt,
                    marked: header.marked,
                    memcat: header.memcat,
                });

                current = header.next;
            } else {
                break;
            }
        }

        Ok(objects)
    }

    pub fn find_all_strings(&self, global_state: Address) -> Result<Vec<StringInfo>, MemoryError> {
        let mut strings = Vec::new();

        Ok(strings)
    }

    pub fn find_all_tables(&self, global_state: Address) -> Result<Vec<TableInfo>, MemoryError> {
        let mut tables = Vec::new();

        Ok(tables)
    }

    pub fn find_all_functions(&self, global_state: Address) -> Result<Vec<FunctionInfo>, MemoryError> {
        let mut functions = Vec::new();

        Ok(functions)
    }

    pub fn get_gc_statistics(&self, global_state: Address) -> Result<GcStatistics, MemoryError> {
        let mut stats = GcStatistics::new();

        Ok(stats)
    }

    pub fn find_gc_roots(&self, state: Address) -> Result<Vec<Address>, MemoryError> {
        let mut roots = Vec::new();

        Ok(roots)
    }

    pub fn trace_references(&self, object: Address, max_depth: usize) -> Result<ReferenceGraph, MemoryError> {
        let mut graph = ReferenceGraph::new();
        let mut queue = vec![(object, 0)];
        let mut visited = HashSet::new();

        while let Some((current, depth)) = queue.pop() {
            if depth > max_depth || !visited.insert(current.as_u64()) {
                continue;
            }

            let header_data = self.reader.read_bytes(current, 16)?;
            if let Some(header) = GCHeader::from_bytes(&header_data) {
                graph.add_node(current, header.tt);

                let refs = self.get_object_references(current, header.tt)?;
                for ref_addr in refs {
                    graph.add_edge(current, ref_addr);
                    queue.push((ref_addr, depth + 1));
                }
            }
        }

        Ok(graph)
    }

    fn get_object_references(&self, addr: Address, tt: TypeTag) -> Result<Vec<Address>, MemoryError> {
        let mut refs = Vec::new();

        match tt {
            TypeTag::Table => {
                let mt = self.reader.read_u64(addr + 0x28)?;
                if mt != 0 {
                    refs.push(Address::new(mt));
                }
            }
            TypeTag::Function => {
                let env = self.reader.read_u64(addr + 0x18)?;
                if env != 0 {
                    refs.push(Address::new(env));
                }
            }
            TypeTag::Thread => {
                let gt = self.reader.read_u64(addr + 0x38)?;
                if gt != 0 {
                    refs.push(Address::new(gt));
                }
            }
            _ => {}
        }

        Ok(refs)
    }
}

#[derive(Debug, Clone)]
pub struct GcStateInfo {
    pub current_white: u8,
    pub gc_state: GcPhase,
    pub sweep_page: Option<Address>,
    pub gray: Option<Address>,
    pub gray_again: Option<Address>,
    pub weak: Option<Address>,
    pub total_bytes: u64,
    pub gc_goal: u64,
    pub gc_step_mul: u32,
    pub gc_step_size: u32,
}

impl GcStateInfo {
    pub fn new() -> Self {
        Self {
            current_white: 0,
            gc_state: GcPhase::Pause,
            sweep_page: None,
            gray: None,
            gray_again: None,
            weak: None,
            total_bytes: 0,
            gc_goal: 0,
            gc_step_mul: 0,
            gc_step_size: 0,
        }
    }
}

impl Default for GcStateInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcPhase {
    Pause,
    Propagate,
    PropagateAgain,
    Sweep,
    SweepStrings,
    Unknown(u8),
}

impl GcPhase {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => GcPhase::Pause,
            1 => GcPhase::Propagate,
            2 => GcPhase::PropagateAgain,
            3 => GcPhase::Sweep,
            4 => GcPhase::SweepStrings,
            _ => GcPhase::Unknown(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GcObjectInfo {
    pub address: Address,
    pub type_tag: TypeTag,
    pub marked: u8,
    pub memcat: u8,
}

impl GcObjectInfo {
    pub fn type_name(&self) -> &'static str {
        match self.type_tag {
            TypeTag::Nil => "nil",
            TypeTag::Boolean => "boolean",
            TypeTag::LightUserData => "lightuserdata",
            TypeTag::Number => "number",
            TypeTag::Vector => "vector",
            TypeTag::String => "string",
            TypeTag::Table => "table",
            TypeTag::Function => "function",
            TypeTag::UserData => "userdata",
            TypeTag::Thread => "thread",
            TypeTag::Unknown(_) => "unknown",
        }
    }

    pub fn is_white(&self) -> bool {
        self.marked & 0x03 != 0
    }

    pub fn is_black(&self) -> bool {
        self.marked & 0x04 != 0
    }

    pub fn is_gray(&self) -> bool {
        !self.is_white() && !self.is_black()
    }
}

#[derive(Debug, Clone)]
pub struct StringInfo {
    pub address: Address,
    pub hash: u32,
    pub len: u32,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub address: Address,
    pub array_size: u32,
    pub node_size: u32,
    pub flags: u8,
    pub has_metatable: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub address: Address,
    pub is_c_function: bool,
    pub nupvalues: u8,
    pub proto_address: Option<Address>,
}

#[derive(Debug, Clone, Default)]
pub struct GcStatistics {
    pub total_objects: usize,
    pub string_count: usize,
    pub table_count: usize,
    pub function_count: usize,
    pub userdata_count: usize,
    pub thread_count: usize,
    pub total_bytes: u64,
    pub string_bytes: u64,
    pub table_bytes: u64,
}

impl GcStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn object_distribution(&self) -> HashMap<&'static str, usize> {
        let mut dist = HashMap::new();
        dist.insert("string", self.string_count);
        dist.insert("table", self.table_count);
        dist.insert("function", self.function_count);
        dist.insert("userdata", self.userdata_count);
        dist.insert("thread", self.thread_count);
        dist
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceGraph {
    nodes: HashMap<u64, TypeTag>,
    edges: HashMap<u64, Vec<u64>>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, addr: Address, tt: TypeTag) {
        self.nodes.insert(addr.as_u64(), tt);
    }

    pub fn add_edge(&mut self, from: Address, to: Address) {
        self.edges.entry(from.as_u64())
            .or_default()
            .push(to.as_u64());
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }

    pub fn get_references(&self, addr: Address) -> Option<&Vec<u64>> {
        self.edges.get(&addr.as_u64())
    }

    pub fn get_type(&self, addr: Address) -> Option<TypeTag> {
        self.nodes.get(&addr.as_u64()).copied()
    }
}

impl Default for ReferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}
