// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::types::{TypeTag, TValue};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Upvalue state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpvalueState {
    /// Upvalue points to stack slot
    Open,
    /// Upvalue has been closed (value copied)
    Closed,
}

/// Represents a Luau upvalue
#[derive(Debug, Clone)]
pub struct Upvalue {
    /// Address of the upvalue structure
    pub address: Address,
    /// Current state
    pub state: UpvalueState,
    /// If open, the stack address; if closed, points to value storage
    pub value_location: Address,
    /// The actual value (if readable)
    pub value: Option<TValue>,
    /// Index in the closure's upvalue array
    pub index: usize,
    /// Name of the upvalue (from debug info)
    pub name: Option<String>,
}

impl Upvalue {
    pub fn new(address: Address, index: usize) -> Self {
        Self {
            address,
            state: UpvalueState::Open,
            value_location: Address::new(0),
            value: None,
            index,
            name: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.state == UpvalueState::Open
    }

    pub fn is_closed(&self) -> bool {
        self.state == UpvalueState::Closed
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

impl fmt::Display for Upvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("unnamed");
        let state = if self.is_open() { "open" } else { "closed" };
        write!(f, "upval[{}] '{}' @ {:016x} ({})", 
            self.index, name, self.address.as_u64(), state)?;
        if let Some(ref value) = self.value {
            write!(f, " = {:?}", value)?;
        }
        Ok(())
    }
}

/// Analyzes upvalues in Luau closures
pub struct UpvalueAnalyzer {
    reader: Arc<dyn MemoryReader>,
    upvalue_size: usize,
}

impl UpvalueAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            upvalue_size: 24, // Typical size
        }
    }

    pub fn with_upvalue_size(mut self, size: usize) -> Self {
        self.upvalue_size = size;
        self
    }

    /// Analyze upvalues of a closure
    pub fn analyze_closure_upvalues(&self, closure_addr: Address) -> Result<Vec<Upvalue>, MemoryError> {
        let mut upvalues = Vec::new();

        // Read closure header to get upvalue count
        // Typical closure layout:
        // +0x00: GCHeader
        // +0x08: env (Table*)
        // +0x10: nupvalues (uint8)
        // +0x18: upvalues array
        
        let nupvalues = self.reader.read_u8(closure_addr + 0x10)?;
        let upvalues_base = closure_addr + 0x18;

        for i in 0..nupvalues as usize {
            let upval_ptr = self.reader.read_u64(upvalues_base + (i * 8) as u64)?;
            if upval_ptr != 0 {
                let upval = self.analyze_upvalue(Address::new(upval_ptr), i)?;
                upvalues.push(upval);
            }
        }

        Ok(upvalues)
    }

    /// Analyze a single upvalue
    pub fn analyze_upvalue(&self, addr: Address, index: usize) -> Result<Upvalue, MemoryError> {
        let mut upval = Upvalue::new(addr, index);

        // Upvalue structure:
        // +0x00: GCHeader
        // +0x08: v (TValue*) - points to value
        // +0x10: u.value (TValue) - storage for closed upvalue
        // +0x20: u.l.prev/next - linked list
        
        let v_ptr = self.reader.read_u64(addr + 0x08)?;
        let storage_addr = addr + 0x10;

        // If v points to storage, upvalue is closed
        if v_ptr == storage_addr.as_u64() {
            upval.state = UpvalueState::Closed;
            upval.value_location = storage_addr;
        } else {
            upval.state = UpvalueState::Open;
            upval.value_location = Address::new(v_ptr);
        }

        // Try to read the value
        if let Ok(value) = self.read_tvalue(upval.value_location) {
            upval.value = Some(value);
        }

        Ok(upval)
    }

    fn read_tvalue(&self, addr: Address) -> Result<TValue, MemoryError> {
        let tt_byte = self.reader.read_u8(addr + 8)?;
        let tt = TypeTag::from_u8(tt_byte);
        
        // Return a basic TValue (nil for now - would need proper parsing)
        Ok(TValue::nil())
    }

    /// Find all open upvalues on a thread's stack
    pub fn find_open_upvalues(&self, lua_state: Address) -> Result<Vec<Upvalue>, MemoryError> {
        let mut upvalues = Vec::new();

        // Read openupval list from lua_State
        // Typically at offset ~0x48
        let openupval_ptr = self.reader.read_u64(lua_state + 0x48)?;
        
        let mut current = Address::new(openupval_ptr);
        let mut visited = HashSet::new();
        
        while current.as_u64() != 0 && !visited.contains(&current.as_u64()) {
            visited.insert(current.as_u64());
            
            let upval = self.analyze_upvalue(current, upvalues.len())?;
            
            // Get next in list (typically at offset 0x20)
            let next_ptr = self.reader.read_u64(current + 0x20)?;
            
            upvalues.push(upval);
            current = Address::new(next_ptr);
        }

        Ok(upvalues)
    }

    /// Track upvalue references across closures
    pub fn track_upvalue_references(&self, closures: &[Address]) -> Result<UpvalueRefMap, MemoryError> {
        let mut ref_map = UpvalueRefMap::new();

        for &closure in closures {
            let upvalues = self.analyze_closure_upvalues(closure)?;
            
            for upval in upvalues {
                ref_map.add_reference(upval.address, closure, upval.index);
            }
        }

        Ok(ref_map)
    }
}

/// Map of upvalue references
#[derive(Debug, Clone)]
pub struct UpvalueRefMap {
    /// Map from upvalue address to (closure, index) pairs
    references: HashMap<u64, Vec<(Address, usize)>>,
}

impl UpvalueRefMap {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
        }
    }

    pub fn add_reference(&mut self, upval_addr: Address, closure: Address, index: usize) {
        self.references
            .entry(upval_addr.as_u64())
            .or_default()
            .push((closure, index));
    }

    pub fn get_references(&self, upval_addr: Address) -> Option<&Vec<(Address, usize)>> {
        self.references.get(&upval_addr.as_u64())
    }

    pub fn reference_count(&self, upval_addr: Address) -> usize {
        self.references.get(&upval_addr.as_u64())
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn shared_upvalues(&self) -> Vec<Address> {
        self.references.iter()
            .filter(|(_, refs)| refs.len() > 1)
            .map(|(&addr, _)| Address::new(addr))
            .collect()
    }

    pub fn unique_upvalue_count(&self) -> usize {
        self.references.len()
    }
}

impl Default for UpvalueRefMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Upvalue scope analyzer
pub struct UpvalueScopeAnalyzer {
    scopes: Vec<UpvalueScope>,
}

impl UpvalueScopeAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
        }
    }

    pub fn analyze_function(&mut self, func_start: usize, func_end: usize, upvalues: &[String]) {
        let scope = UpvalueScope {
            start_pc: func_start,
            end_pc: func_end,
            upvalues: upvalues.to_vec(),
            captures: Vec::new(),
        };
        self.scopes.push(scope);
    }

    pub fn add_capture(&mut self, scope_index: usize, local_name: &str, upvalue_index: usize) {
        if let Some(scope) = self.scopes.get_mut(scope_index) {
            scope.captures.push(UpvalueCapture {
                local_name: local_name.to_string(),
                upvalue_index,
            });
        }
    }

    pub fn get_scope(&self, pc: usize) -> Option<&UpvalueScope> {
        self.scopes.iter()
            .find(|s| pc >= s.start_pc && pc <= s.end_pc)
    }
}

impl Default for UpvalueScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a scope with upvalue information
#[derive(Debug, Clone)]
pub struct UpvalueScope {
    pub start_pc: usize,
    pub end_pc: usize,
    pub upvalues: Vec<String>,
    pub captures: Vec<UpvalueCapture>,
}

/// Information about an upvalue capture
#[derive(Debug, Clone)]
pub struct UpvalueCapture {
    pub local_name: String,
    pub upvalue_index: usize,
}

/// Tracks upvalue lifetime
pub struct UpvalueLifetime {
    /// PC where upvalue is first captured
    pub capture_pc: usize,
    /// PC where upvalue is closed (if known)
    pub close_pc: Option<usize>,
    /// All PCs where upvalue is read
    pub reads: Vec<usize>,
    /// All PCs where upvalue is written
    pub writes: Vec<usize>,
}

impl UpvalueLifetime {
    pub fn new(capture_pc: usize) -> Self {
        Self {
            capture_pc,
            close_pc: None,
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub fn add_read(&mut self, pc: usize) {
        self.reads.push(pc);
    }

    pub fn add_write(&mut self, pc: usize) {
        self.writes.push(pc);
    }

    pub fn set_close(&mut self, pc: usize) {
        self.close_pc = Some(pc);
    }

    pub fn is_read_only(&self) -> bool {
        self.writes.is_empty()
    }

    pub fn usage_count(&self) -> usize {
        self.reads.len() + self.writes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upvalue_state() {
        let mut upval = Upvalue::new(Address::new(0x1000), 0);
        assert!(upval.is_open());
        
        upval.state = UpvalueState::Closed;
        assert!(upval.is_closed());
    }

    #[test]
    fn test_upvalue_ref_map() {
        let mut map = UpvalueRefMap::new();
        
        map.add_reference(Address::new(0x1000), Address::new(0x2000), 0);
        map.add_reference(Address::new(0x1000), Address::new(0x3000), 1);
        
        assert_eq!(map.reference_count(Address::new(0x1000)), 2);
        assert_eq!(map.shared_upvalues().len(), 1);
    }

    #[test]
    fn test_upvalue_lifetime() {
        let mut lifetime = UpvalueLifetime::new(10);
        lifetime.add_read(15);
        lifetime.add_read(20);
        lifetime.add_write(25);
        
        assert!(!lifetime.is_read_only());
        assert_eq!(lifetime.usage_count(), 3);
    }
}
