// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents a data definition (where a value is assigned)
#[derive(Debug, Clone)]
pub struct DataDefinition {
    /// Address where the definition occurs
    pub address: Address,
    /// Register or memory location being defined
    pub location: DataLocation,
    /// The value or expression being assigned (if known)
    pub value: Option<DataValue>,
    /// Basic block containing this definition
    pub block_id: u64,
}

impl DataDefinition {
    pub fn new(address: Address, location: DataLocation) -> Self {
        Self {
            address,
            location,
            value: None,
            block_id: 0,
        }
    }

    pub fn with_value(mut self, value: DataValue) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_block(mut self, block_id: u64) -> Self {
        self.block_id = block_id;
        self
    }
}

impl fmt::Display for DataDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}: {} = ", self.address.as_u64(), self.location)?;
        if let Some(ref value) = self.value {
            write!(f, "{}", value)
        } else {
            write!(f, "?")
        }
    }
}

/// Represents a data use (where a value is read)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataUse {
    /// Address where the use occurs
    pub address: Address,
    /// Register or memory location being used
    pub location: DataLocation,
    /// Basic block containing this use
    pub block_id: u64,
}

impl DataUse {
    pub fn new(address: Address, location: DataLocation) -> Self {
        Self {
            address,
            location,
            block_id: 0,
        }
    }

    pub fn with_block(mut self, block_id: u64) -> Self {
        self.block_id = block_id;
        self
    }
}

impl fmt::Display for DataUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}: use {}", self.address.as_u64(), self.location)
    }
}

/// Represents a location that can hold data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataLocation {
    /// CPU register (x0-x30 on ARM64)
    Register(u8),
    /// Stack slot at offset from SP/FP
    Stack(i64),
    /// Global memory address
    Memory(u64),
    /// Heap allocation (tracked by allocation site)
    Heap(u64),
    /// Unknown location
    Unknown,
}

impl DataLocation {
    pub fn reg(num: u8) -> Self {
        DataLocation::Register(num)
    }

    pub fn stack(offset: i64) -> Self {
        DataLocation::Stack(offset)
    }

    pub fn memory(addr: u64) -> Self {
        DataLocation::Memory(addr)
    }

    pub fn is_register(&self) -> bool {
        matches!(self, DataLocation::Register(_))
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, DataLocation::Memory(_) | DataLocation::Stack(_) | DataLocation::Heap(_))
    }
}

impl fmt::Display for DataLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataLocation::Register(r) => write!(f, "x{}", r),
            DataLocation::Stack(off) => {
                if *off >= 0 {
                    write!(f, "[sp+0x{:x}]", off)
                } else {
                    write!(f, "[sp-0x{:x}]", -off)
                }
            }
            DataLocation::Memory(addr) => write!(f, "[0x{:x}]", addr),
            DataLocation::Heap(site) => write!(f, "heap@0x{:x}", site),
            DataLocation::Unknown => write!(f, "?"),
        }
    }
}

/// Represents a data value (constant or expression)
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    /// Constant integer value
    Constant(i64),
    /// Address/pointer value
    Address(u64),
    /// Value from another location
    Copy(DataLocation),
    /// Computed value (operation + operands)
    Computed(DataOperation, Vec<DataLocation>),
    /// Function return value
    ReturnValue(u64), // function address
    /// Unknown value
    Unknown,
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::Constant(v) => write!(f, "0x{:x}", v),
            DataValue::Address(a) => write!(f, "&0x{:x}", a),
            DataValue::Copy(loc) => write!(f, "{}", loc),
            DataValue::Computed(op, operands) => {
                let ops: Vec<String> = operands.iter().map(|o| format!("{}", o)).collect();
                write!(f, "{}({})", op, ops.join(", "))
            }
            DataValue::ReturnValue(func) => write!(f, "ret@0x{:x}", func),
            DataValue::Unknown => write!(f, "?"),
        }
    }
}

/// Data operations for computed values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataOperation {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Load,
    Store,
    Call,
    Phi,
}

impl fmt::Display for DataOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataOperation::Add => write!(f, "add"),
            DataOperation::Sub => write!(f, "sub"),
            DataOperation::Mul => write!(f, "mul"),
            DataOperation::Div => write!(f, "div"),
            DataOperation::And => write!(f, "and"),
            DataOperation::Or => write!(f, "or"),
            DataOperation::Xor => write!(f, "xor"),
            DataOperation::Shl => write!(f, "shl"),
            DataOperation::Shr => write!(f, "shr"),
            DataOperation::Load => write!(f, "load"),
            DataOperation::Store => write!(f, "store"),
            DataOperation::Call => write!(f, "call"),
            DataOperation::Phi => write!(f, "φ"),
        }
    }
}

/// Def-Use chain connecting definitions to their uses
#[derive(Debug, Clone)]
pub struct DefUseChain {
    /// The definition
    pub definition: DataDefinition,
    /// All uses of this definition
    pub uses: Vec<DataUse>,
}

impl DefUseChain {
    pub fn new(definition: DataDefinition) -> Self {
        Self {
            definition,
            uses: Vec::new(),
        }
    }

    pub fn add_use(&mut self, use_: DataUse) {
        self.uses.push(use_);
    }

    pub fn use_count(&self) -> usize {
        self.uses.len()
    }

    pub fn is_dead(&self) -> bool {
        self.uses.is_empty()
    }
}

impl fmt::Display for DefUseChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Definition: {}", self.definition)?;
        writeln!(f, "Uses ({}):", self.uses.len())?;
        for use_ in &self.uses {
            writeln!(f, "  {}", use_)?;
        }
        Ok(())
    }
}

/// Use-Def chain connecting uses to their definitions
#[derive(Debug, Clone)]
pub struct UseDefChain {
    /// The use
    pub use_: DataUse,
    /// All possible definitions reaching this use
    pub definitions: Vec<DataDefinition>,
}

impl UseDefChain {
    pub fn new(use_: DataUse) -> Self {
        Self {
            use_,
            definitions: Vec::new(),
        }
    }

    pub fn add_definition(&mut self, def: DataDefinition) {
        self.definitions.push(def);
    }

    pub fn def_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_undefined(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn has_single_definition(&self) -> bool {
        self.definitions.len() == 1
    }
}

impl fmt::Display for UseDefChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Use: {}", self.use_)?;
        writeln!(f, "Definitions ({}):", self.definitions.len())?;
        for def in &self.definitions {
            writeln!(f, "  {}", def)?;
        }
        Ok(())
    }
}

/// Data flow analyzer for tracking value flow through code
pub struct DataFlowAnalyzer {
    reader: Arc<dyn MemoryReader>,
    /// All definitions found
    definitions: Vec<DataDefinition>,
    /// All uses found
    uses: Vec<DataUse>,
    /// Def-Use chains
    def_use_chains: HashMap<(u64, DataLocation), DefUseChain>,
    /// Use-Def chains
    use_def_chains: HashMap<(u64, DataLocation), UseDefChain>,
    /// Reaching definitions at each program point
    reaching_defs: HashMap<u64, Vec<DataDefinition>>,
    /// Live variables at each program point
    live_vars: HashMap<u64, HashSet<DataLocation>>,
}

impl DataFlowAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            definitions: Vec::new(),
            uses: Vec::new(),
            def_use_chains: HashMap::new(),
            use_def_chains: HashMap::new(),
            reaching_defs: HashMap::new(),
            live_vars: HashMap::new(),
        }
    }

    /// Analyze a function for data flow
    pub fn analyze_function(&mut self, start: Address, _end: Address) -> Result<DataFlowResult, MemoryError> {
        // Clear previous analysis
        self.definitions.clear();
        self.uses.clear();
        self.def_use_chains.clear();
        self.use_def_chains.clear();

        // First pass: collect definitions and uses
        self.collect_defs_uses(start)?;

        // Build def-use chains
        self.build_def_use_chains();

        // Build use-def chains
        self.build_use_def_chains();

        // Compute reaching definitions
        self.compute_reaching_definitions();

        // Compute live variables
        self.compute_live_variables();

        Ok(DataFlowResult {
            definitions: self.definitions.clone(),
            uses: self.uses.clone(),
            def_use_chains: self.def_use_chains.values().cloned().collect(),
            use_def_chains: self.use_def_chains.values().cloned().collect(),
            dead_definitions: self.find_dead_definitions(),
            undefined_uses: self.find_undefined_uses(),
        })
    }

    /// Collect definitions and uses from instructions
    fn collect_defs_uses(&mut self, start: Address) -> Result<(), MemoryError> {
        let mut addr = start;
        let max_instructions = 1000;
        
        for _ in 0..max_instructions {
            let bytes = self.reader.read_bytes(addr, 4)?;
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            // Parse ARM64 instruction for defs and uses
            self.parse_instruction_defs_uses(addr, insn);

            // Check for function end
            if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                break; // RET
            }

            addr = addr + 4;
        }

        Ok(())
    }

    /// Parse an ARM64 instruction for definitions and uses
    fn parse_instruction_defs_uses(&mut self, addr: Address, insn: u32) {
        let op = insn >> 24;

        match op {
            // LDR - load defines a register, uses memory
            0xF9 | 0xB9 | 0x39 => {
                let rt = (insn & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                
                self.definitions.push(DataDefinition::new(addr, DataLocation::reg(rt)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
            }
            // STR - store uses two registers
            0xF8 | 0xB8 | 0x38 => {
                let rt = (insn & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                
                self.uses.push(DataUse::new(addr, DataLocation::reg(rt)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
            }
            // ADD/SUB immediate - defines rd, uses rn
            0x91 | 0xD1 | 0x11 | 0x51 => {
                let rd = (insn & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                
                self.definitions.push(DataDefinition::new(addr, DataLocation::reg(rd)));
                if rd != rn {
                    self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
                }
            }
            // ADD/SUB register - defines rd, uses rn and rm
            0x8B | 0xCB | 0x0B | 0x4B => {
                let rd = (insn & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                let rm = ((insn >> 16) & 0x1F) as u8;
                
                self.definitions.push(DataDefinition::new(addr, DataLocation::reg(rd)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rm)));
            }
            // MOV (ORR) - defines rd, uses rm
            0xAA | 0x2A => {
                let rd = (insn & 0x1F) as u8;
                let rm = ((insn >> 16) & 0x1F) as u8;
                
                self.definitions.push(
                    DataDefinition::new(addr, DataLocation::reg(rd))
                        .with_value(DataValue::Copy(DataLocation::reg(rm)))
                );
                if rm != 31 { // Not ZR
                    self.uses.push(DataUse::new(addr, DataLocation::reg(rm)));
                }
            }
            // MOVZ/MOVK - defines rd with immediate
            0xD2 | 0xF2 | 0x52 | 0x72 => {
                let rd = (insn & 0x1F) as u8;
                let imm16 = ((insn >> 5) & 0xFFFF) as i64;
                let hw = ((insn >> 21) & 0x3) as i64;
                let value = imm16 << (hw * 16);
                
                self.definitions.push(
                    DataDefinition::new(addr, DataLocation::reg(rd))
                        .with_value(DataValue::Constant(value))
                );
            }
            // BL - call modifies x0 (return value) and uses function address
            0x94 | 0x97 => {
                // Function calls define x0 (return value) and clobber caller-saved regs
                self.definitions.push(
                    DataDefinition::new(addr, DataLocation::reg(0))
                        .with_value(DataValue::ReturnValue(0)) // Would need target address
                );
                // Caller-saved registers are clobbered
                for r in 0..18 {
                    self.definitions.push(DataDefinition::new(addr, DataLocation::reg(r)));
                }
            }
            // STP - store pair
            0xA9 | 0x29 => {
                let rt = (insn & 0x1F) as u8;
                let rt2 = ((insn >> 10) & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                
                self.uses.push(DataUse::new(addr, DataLocation::reg(rt)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rt2)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
            }
            // LDP - load pair
            0xA8 | 0x28 => {
                let rt = (insn & 0x1F) as u8;
                let rt2 = ((insn >> 10) & 0x1F) as u8;
                let rn = ((insn >> 5) & 0x1F) as u8;
                
                self.definitions.push(DataDefinition::new(addr, DataLocation::reg(rt)));
                self.definitions.push(DataDefinition::new(addr, DataLocation::reg(rt2)));
                self.uses.push(DataUse::new(addr, DataLocation::reg(rn)));
            }
            _ => {}
        }
    }

    /// Build def-use chains from collected data
    fn build_def_use_chains(&mut self) {
        for def in &self.definitions {
            let key = (def.address.as_u64(), def.location.clone());
            let mut chain = DefUseChain::new(def.clone());

            // Find all uses of this definition (simplified - would need control flow)
            for use_ in &self.uses {
                if use_.location == def.location && use_.address.as_u64() > def.address.as_u64() {
                    chain.add_use(use_.clone());
                }
            }

            self.def_use_chains.insert(key, chain);
        }
    }

    /// Build use-def chains from collected data
    fn build_use_def_chains(&mut self) {
        for use_ in &self.uses {
            let key = (use_.address.as_u64(), use_.location.clone());
            let mut chain = UseDefChain::new(use_.clone());

            // Find all definitions that could reach this use
            for def in &self.definitions {
                if def.location == use_.location && def.address.as_u64() < use_.address.as_u64() {
                    chain.add_definition(def.clone());
                }
            }

            self.use_def_chains.insert(key, chain);
        }
    }

    /// Compute reaching definitions using iterative data flow analysis
    fn compute_reaching_definitions(&mut self) {
        // Simplified implementation - would need proper CFG for accuracy
        let mut reaching: HashMap<u64, Vec<DataDefinition>> = HashMap::new();

        for def in &self.definitions {
            reaching.entry(def.address.as_u64())
                .or_default()
                .push(def.clone());
        }

        self.reaching_defs = reaching;
    }

    /// Compute live variables using backward data flow analysis
    fn compute_live_variables(&mut self) {
        // Simplified implementation
        let mut live: HashMap<u64, HashSet<DataLocation>> = HashMap::new();

        for use_ in &self.uses {
            live.entry(use_.address.as_u64())
                .or_default()
                .insert(use_.location.clone());
        }

        self.live_vars = live;
    }

    /// Find definitions that are never used (dead code)
    fn find_dead_definitions(&self) -> Vec<DataDefinition> {
        self.def_use_chains.values()
            .filter(|chain| chain.is_dead())
            .map(|chain| chain.definition.clone())
            .collect()
    }

    /// Find uses without any reaching definitions
    fn find_undefined_uses(&self) -> Vec<DataUse> {
        self.use_def_chains.values()
            .filter(|chain| chain.is_undefined())
            .map(|chain| chain.use_.clone())
            .collect()
    }

    /// Get reaching definitions at a specific address
    pub fn get_reaching_defs(&self, addr: Address) -> Vec<&DataDefinition> {
        self.reaching_defs.get(&addr.as_u64())
            .map(|defs| defs.iter().collect())
            .unwrap_or_default()
    }

    /// Get live variables at a specific address
    pub fn get_live_vars(&self, addr: Address) -> Vec<&DataLocation> {
        self.live_vars.get(&addr.as_u64())
            .map(|vars| vars.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a variable is live at a given point
    pub fn is_live_at(&self, location: &DataLocation, addr: Address) -> bool {
        self.live_vars.get(&addr.as_u64())
            .map(|vars| vars.contains(location))
            .unwrap_or(false)
    }
}

/// Result of data flow analysis
#[derive(Debug, Clone)]
pub struct DataFlowResult {
    /// All definitions found
    pub definitions: Vec<DataDefinition>,
    /// All uses found
    pub uses: Vec<DataUse>,
    /// Def-Use chains
    pub def_use_chains: Vec<DefUseChain>,
    /// Use-Def chains
    pub use_def_chains: Vec<UseDefChain>,
    /// Definitions that are never used
    pub dead_definitions: Vec<DataDefinition>,
    /// Uses without definitions
    pub undefined_uses: Vec<DataUse>,
}

impl DataFlowResult {
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn use_count(&self) -> usize {
        self.uses.len()
    }

    pub fn dead_code_count(&self) -> usize {
        self.dead_definitions.len()
    }

    pub fn has_undefined_uses(&self) -> bool {
        !self.undefined_uses.is_empty()
    }

    /// Get statistics about the data flow
    pub fn statistics(&self) -> DataFlowStats {
        DataFlowStats {
            total_definitions: self.definitions.len(),
            total_uses: self.uses.len(),
            dead_definitions: self.dead_definitions.len(),
            undefined_uses: self.undefined_uses.len(),
            def_use_chains: self.def_use_chains.len(),
            avg_uses_per_def: if self.definitions.is_empty() {
                0.0
            } else {
                self.uses.len() as f64 / self.definitions.len() as f64
            },
        }
    }
}

impl fmt::Display for DataFlowResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.statistics();
        writeln!(f, "Data Flow Analysis Result:")?;
        writeln!(f, "  Definitions: {}", stats.total_definitions)?;
        writeln!(f, "  Uses: {}", stats.total_uses)?;
        writeln!(f, "  Dead definitions: {}", stats.dead_definitions)?;
        writeln!(f, "  Undefined uses: {}", stats.undefined_uses)?;
        writeln!(f, "  Def-Use chains: {}", stats.def_use_chains)?;
        writeln!(f, "  Avg uses per def: {:.2}", stats.avg_uses_per_def)?;
        Ok(())
    }
}

/// Statistics from data flow analysis
#[derive(Debug, Clone)]
pub struct DataFlowStats {
    pub total_definitions: usize,
    pub total_uses: usize,
    pub dead_definitions: usize,
    pub undefined_uses: usize,
    pub def_use_chains: usize,
    pub avg_uses_per_def: f64,
}

/// Tracks value propagation through the program
pub struct ValueTracker {
    /// Current known values for locations
    values: HashMap<DataLocation, DataValue>,
    /// History of value changes
    history: Vec<(Address, DataLocation, DataValue)>,
}

impl ValueTracker {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn set_value(&mut self, addr: Address, loc: DataLocation, value: DataValue) {
        self.history.push((addr, loc.clone(), value.clone()));
        self.values.insert(loc, value);
    }

    pub fn get_value(&self, loc: &DataLocation) -> Option<&DataValue> {
        self.values.get(loc)
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn snapshot(&self) -> HashMap<DataLocation, DataValue> {
        self.values.clone()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for ValueTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_location_display() {
        assert_eq!(format!("{}", DataLocation::reg(0)), "x0");
        assert_eq!(format!("{}", DataLocation::stack(-16)), "[sp-0x10]");
        assert_eq!(format!("{}", DataLocation::memory(0x100000)), "[0x100000]");
    }

    #[test]
    fn test_def_use_chain() {
        let def = DataDefinition::new(Address::new(0x1000), DataLocation::reg(0));
        let mut chain = DefUseChain::new(def);
        
        assert!(chain.is_dead());
        
        chain.add_use(DataUse::new(Address::new(0x1004), DataLocation::reg(0)));
        assert!(!chain.is_dead());
        assert_eq!(chain.use_count(), 1);
    }
}
