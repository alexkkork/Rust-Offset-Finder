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

        // Read GC state fields from global_state
        // Typical Luau global_state offsets:
        // currentwhite: offset 0x08 (u8)
        // gcstate: offset 0x09 (u8)
        // sweepgcopage: offset 0x40 (pointer)
        // gray: offset 0x48 (pointer)
        // grayagain: offset 0x50 (pointer)
        // weak: offset 0x58 (pointer)
        // totalbytes: offset 0x80 (u64)
        // gcgoal: offset 0x88 (u64)
        // gcstepmul: offset 0x90 (u32)
        // gcstepsize: offset 0x94 (u32)

        info.current_white = self.reader.read_u8(global_state + 0x08)?;
        let gc_state_byte = self.reader.read_u8(global_state + 0x09)?;
        info.gc_state = GcPhase::from_u8(gc_state_byte);

        let sweep_page = self.reader.read_u64(global_state + 0x40)?;
        if sweep_page != 0 {
            info.sweep_page = Some(Address::new(sweep_page));
        }

        let gray = self.reader.read_u64(global_state + 0x48)?;
        if gray != 0 {
            info.gray = Some(Address::new(gray));
        }

        let gray_again = self.reader.read_u64(global_state + 0x50)?;
        if gray_again != 0 {
            info.gray_again = Some(Address::new(gray_again));
        }

        let weak = self.reader.read_u64(global_state + 0x58)?;
        if weak != 0 {
            info.weak = Some(Address::new(weak));
        }

        info.total_bytes = self.reader.read_u64(global_state + 0x80)?;
        info.gc_goal = self.reader.read_u64(global_state + 0x88)?;
        info.gc_step_mul = self.reader.read_u32(global_state + 0x90)?;
        info.gc_step_size = self.reader.read_u32(global_state + 0x94)?;

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

        // Walk the string table in global_state
        // strt.hash: offset 0x100 (pointer to string hash table)
        // strt.nuse: offset 0x108 (u32 - number of strings in use)
        // strt.size: offset 0x10C (u32 - size of hash table)

        let string_table_ptr = self.reader.read_u64(global_state + 0x100)?;
        let num_strings = self.reader.read_u32(global_state + 0x108)? as usize;
        let table_size = self.reader.read_u32(global_state + 0x10C)? as usize;

        if string_table_ptr == 0 || table_size == 0 {
            return Ok(strings);
        }

        // Each entry in the hash table is a pointer to a string linked list
        for i in 0..table_size.min(10000) {
            let entry_addr = Address::new(string_table_ptr) + (i as u64 * 8);
            let string_ptr = self.reader.read_u64(entry_addr)?;

            if string_ptr == 0 {
                continue;
            }

            // Walk the linked list of strings at this hash bucket
            let mut current = Address::new(string_ptr);
            let mut visited = HashSet::new();

            while current.as_u64() != 0 && visited.len() < 1000 {
                if !visited.insert(current.as_u64()) {
                    break;
                }

                // Read string header: GCHeader + atom(u16) + hash(u32) + len(u32)
                let header_data = self.reader.read_bytes(current, 24)?;
                if let Some(gc_header) = GCHeader::from_bytes(&header_data) {
                    if gc_header.tt == TypeTag::String {
                        let hash = u32::from_le_bytes([header_data[16], header_data[17], header_data[18], header_data[19]]);
                        let len = u32::from_le_bytes([header_data[20], header_data[21], header_data[22], header_data[23]]);

                        // Try to read string content
                        let content = if len > 0 && len < 1024 {
                            let data_addr = current + 0x18;
                            if let Ok(bytes) = self.reader.read_bytes(data_addr, len as usize) {
                                String::from_utf8(bytes).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        strings.push(StringInfo {
                            address: current,
                            hash,
                            len,
                            content,
                        });
                    }

                    current = gc_header.next;
                } else {
                    break;
                }

                if strings.len() >= num_strings.max(10000) {
                    break;
                }
            }

            if strings.len() >= num_strings.max(10000) {
                break;
            }
        }

        Ok(strings)
    }

    pub fn find_all_tables(&self, global_state: Address) -> Result<Vec<TableInfo>, MemoryError> {
        let mut tables = Vec::new();

        // Walk all GC objects and filter for tables
        // allgcopages: offset 0x38 (pointer to first GC page)
        let allgco = self.reader.read_u64(global_state + 0x38)?;
        if allgco == 0 {
            return Ok(tables);
        }

        let gc_objects = self.walk_gc_list(Address::new(allgco), 50000)?;

        for obj in gc_objects {
            if obj.type_tag == TypeTag::Table {
                // Read table-specific fields
                // flags: offset 0x10 (u8)
                // node_log2_size: offset 0x11 (u8)
                // array_size: offset 0x14 (u32)
                // metatable: offset 0x28 (pointer)

                let flags = self.reader.read_u8(obj.address + 0x10)?;
                let node_log2 = self.reader.read_u8(obj.address + 0x11)?;
                let array_size = self.reader.read_u32(obj.address + 0x14)?;
                let metatable = self.reader.read_u64(obj.address + 0x28)?;

                tables.push(TableInfo {
                    address: obj.address,
                    array_size,
                    node_size: 1 << node_log2,
                    flags,
                    has_metatable: metatable != 0,
                });
            }
        }

        Ok(tables)
    }

    pub fn find_all_functions(&self, global_state: Address) -> Result<Vec<FunctionInfo>, MemoryError> {
        let mut functions = Vec::new();

        // Walk all GC objects and filter for functions (closures)
        let allgco = self.reader.read_u64(global_state + 0x38)?;
        if allgco == 0 {
            return Ok(functions);
        }

        let gc_objects = self.walk_gc_list(Address::new(allgco), 50000)?;

        for obj in gc_objects {
            if obj.type_tag == TypeTag::Function {
                // Read closure-specific fields
                // isC: offset 0x10 (u8 - 0 for Lua closure, 1 for C closure)
                // nupvalues: offset 0x11 (u8)
                // For Lua closures: proto at offset 0x20 (pointer)
                // For C closures: f at offset 0x18 (function pointer)

                let is_c = self.reader.read_u8(obj.address + 0x10)? != 0;
                let nupvalues = self.reader.read_u8(obj.address + 0x11)?;

                let proto_address = if !is_c {
                    let proto = self.reader.read_u64(obj.address + 0x20)?;
                    if proto != 0 {
                        Some(Address::new(proto))
                    } else {
                        None
                    }
                } else {
                    None
                };

                functions.push(FunctionInfo {
                    address: obj.address,
                    is_c_function: is_c,
                    nupvalues,
                    proto_address,
                });
            }
        }

        Ok(functions)
    }

    pub fn get_gc_statistics(&self, global_state: Address) -> Result<GcStatistics, MemoryError> {
        let mut stats = GcStatistics::new();

        // Walk all GC objects and count by type
        let allgco = self.reader.read_u64(global_state + 0x38)?;
        if allgco == 0 {
            return Ok(stats);
        }

        let gc_objects = self.walk_gc_list(Address::new(allgco), 100000)?;

        stats.total_objects = gc_objects.len();

        for obj in gc_objects {
            match obj.type_tag {
                TypeTag::String => stats.string_count += 1,
                TypeTag::Table => stats.table_count += 1,
                TypeTag::Function => stats.function_count += 1,
                TypeTag::UserData => stats.userdata_count += 1,
                TypeTag::Thread => stats.thread_count += 1,
                _ => {}
            }
        }

        // Read total bytes from global_state
        stats.total_bytes = self.reader.read_u64(global_state + 0x80)?;

        // Estimate string and table bytes (rough approximation)
        stats.string_bytes = (stats.string_count as u64) * 64; // Average string size estimate
        stats.table_bytes = (stats.table_count as u64) * 128; // Average table size estimate

        Ok(stats)
    }

    pub fn find_gc_roots(&self, state: Address) -> Result<Vec<Address>, MemoryError> {
        let mut roots = Vec::new();

        // GC roots include:
        // 1. Registry table (from global_state)
        // 2. Global table
        // 3. Stack values
        // 4. Upvalues
        // 5. Gray objects (marked but not yet scanned)

        // Get global_state from lua_State
        let global_state_addr = self.reader.read_u64(state + 0x38)?;
        if global_state_addr == 0 {
            return Ok(roots);
        }
        let global_state = Address::new(global_state_addr);

        // Registry table: typically at offset 0x60 in global_state
        let registry = self.reader.read_u64(global_state + 0x60)?;
        if registry != 0 {
            roots.push(Address::new(registry));
        }

        // Main thread (from global_state at offset 0x08)
        let main_thread = self.reader.read_u64(global_state + 0x00)?;
        if main_thread != 0 {
            roots.push(Address::new(main_thread));
        }

        // Global table (gt) from the lua_State
        let gt = self.reader.read_u64(state + 0x40)?;
        if gt != 0 {
            roots.push(Address::new(gt));
        }

        // Stack values from current state
        let stack_base = self.reader.read_u64(state + 0x18)?;
        let stack_top = self.reader.read_u64(state + 0x10)?;

        if stack_base != 0 && stack_top != 0 && stack_top > stack_base {
            let stack_size = ((stack_top - stack_base) / 16) as usize;
            for i in 0..stack_size.min(1000) {
                let value_addr = stack_base + (i as u64 * 16);
                let tt = self.reader.read_u8(Address::new(value_addr) + 8)?;

                // Check if this is a GC-collectable type (5-9: string, table, function, userdata, thread)
                if tt >= 5 && tt <= 9 {
                    let gc_obj = self.reader.read_u64(Address::new(value_addr))?;
                    if gc_obj != 0 {
                        roots.push(Address::new(gc_obj));
                    }
                }
            }
        }

        // Gray list (objects pending traversal)
        let gray = self.reader.read_u64(global_state + 0x48)?;
        if gray != 0 {
            roots.push(Address::new(gray));
        }

        // Gray again list
        let gray_again = self.reader.read_u64(global_state + 0x50)?;
        if gray_again != 0 {
            roots.push(Address::new(gray_again));
        }

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
