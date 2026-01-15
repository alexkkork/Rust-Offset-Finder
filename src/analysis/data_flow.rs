// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::{BasicBlock, ControlFlowGraph, Instruction};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct DataFlowAnalyzer {
    cfg: ControlFlowGraph,
    live_in: HashMap<u64, HashSet<u8>>,
    live_out: HashMap<u64, HashSet<u8>>,
    reaching_defs: HashMap<u64, HashSet<(u64, u8)>>,
    available_exprs: HashMap<u64, HashSet<u64>>,
}

impl DataFlowAnalyzer {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        Self {
            cfg,
            live_in: HashMap::new(),
            live_out: HashMap::new(),
            reaching_defs: HashMap::new(),
            available_exprs: HashMap::new(),
        }
    }

    pub fn compute_liveness(&mut self) {
        self.live_in.clear();
        self.live_out.clear();

        for block in self.cfg.blocks() {
            self.live_in.insert(block.start().as_u64(), HashSet::new());
            self.live_out.insert(block.start().as_u64(), HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in self.cfg.blocks() {
                let addr = block.start().as_u64();

                let mut new_out = HashSet::new();
                for succ in block.successors() {
                    if let Some(succ_in) = self.live_in.get(&succ.as_u64()) {
                        new_out.extend(succ_in);
                    }
                }

                let old_out = self.live_out.get(&addr).cloned().unwrap_or_default();
                if new_out != old_out {
                    self.live_out.insert(addr, new_out.clone());
                    changed = true;
                }

                let data_flow = block.analyze_data_flow();
                let mut new_in = new_out.clone();

                for &killed in &data_flow.killed {
                    new_in.remove(&killed);
                }

                for &used in &data_flow.used {
                    new_in.insert(used);
                }

                let old_in = self.live_in.get(&addr).cloned().unwrap_or_default();
                if new_in != old_in {
                    self.live_in.insert(addr, new_in);
                    changed = true;
                }
            }
        }
    }

    pub fn compute_reaching_definitions(&mut self) {
        self.reaching_defs.clear();

        for block in self.cfg.blocks() {
            self.reaching_defs.insert(block.start().as_u64(), HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in self.cfg.blocks() {
                let addr = block.start().as_u64();

                let mut new_in = HashSet::new();
                for pred in block.predecessors() {
                    if let Some(pred_out) = self.get_reaching_out(*pred) {
                        new_in.extend(pred_out);
                    }
                }

                let old_in = self.reaching_defs.get(&addr).cloned().unwrap_or_default();
                if new_in != old_in {
                    self.reaching_defs.insert(addr, new_in);
                    changed = true;
                }
            }
        }
    }

    fn get_reaching_out(&self, addr: Address) -> Option<HashSet<(u64, u8)>> {
        let block = self.cfg.get_block(addr)?;
        let mut out = self.reaching_defs.get(&addr.as_u64())?.clone();

        for insn in block.instructions() {
            if let Some(def_reg) = insn.destination_register() {
                out.retain(|(_, reg)| *reg != def_reg);
                out.insert((insn.address().as_u64(), def_reg));
            }
        }

        Some(out)
    }

    pub fn is_live_at(&self, reg: u8, addr: Address) -> bool {
        self.live_in
            .get(&addr.as_u64())
            .map(|set| set.contains(&reg))
            .unwrap_or(false)
    }

    pub fn is_live_out(&self, reg: u8, addr: Address) -> bool {
        self.live_out
            .get(&addr.as_u64())
            .map(|set| set.contains(&reg))
            .unwrap_or(false)
    }

    pub fn live_registers_at(&self, addr: Address) -> HashSet<u8> {
        self.live_in.get(&addr.as_u64()).cloned().unwrap_or_default()
    }

    pub fn reaching_definitions_at(&self, addr: Address) -> HashSet<(u64, u8)> {
        self.reaching_defs.get(&addr.as_u64()).cloned().unwrap_or_default()
    }

    pub fn definitions_of(&self, reg: u8, addr: Address) -> Vec<Address> {
        self.reaching_defs
            .get(&addr.as_u64())
            .map(|defs| {
                defs.iter()
                    .filter(|(_, r)| *r == reg)
                    .map(|(a, _)| Address::new(*a))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn uses_of(&self, def_addr: Address, def_reg: u8) -> Vec<Address> {
        let mut uses = Vec::new();

        for block in self.cfg.blocks() {
            for insn in block.instructions() {
                if insn.uses_register(def_reg) {
                    let defs = self.definitions_of(def_reg, insn.address());
                    if defs.contains(&def_addr) {
                        uses.push(insn.address());
                    }
                }
            }
        }

        uses
    }

    pub fn is_dead_code(&self, addr: Address) -> bool {
        if let Some(block) = self.cfg.get_block(addr) {
            for insn in block.instructions() {
                if insn.address() == addr {
                    if let Some(def_reg) = insn.destination_register() {
                        return !self.is_live_out(def_reg, addr);
                    }
                }
            }
        }
        false
    }

    pub fn find_dead_stores(&self) -> Vec<Address> {
        let mut dead_stores = Vec::new();

        for block in self.cfg.blocks() {
            for insn in block.instructions() {
                if insn.is_store() {
                    let uses = self.uses_of(insn.address(), 0);
                    if uses.is_empty() {
                        dead_stores.push(insn.address());
                    }
                }
            }
        }

        dead_stores
    }

    pub fn compute_def_use_chains(&self) -> HashMap<u64, Vec<u64>> {
        let mut chains = HashMap::new();

        for block in self.cfg.blocks() {
            for insn in block.instructions() {
                if let Some(def_reg) = insn.destination_register() {
                    let uses = self.uses_of(insn.address(), def_reg);
                    chains.insert(
                        insn.address().as_u64(),
                        uses.iter().map(|a| a.as_u64()).collect(),
                    );
                }
            }
        }

        chains
    }

    pub fn compute_use_def_chains(&self) -> HashMap<u64, Vec<u64>> {
        let mut chains = HashMap::new();

        for block in self.cfg.blocks() {
            for insn in block.instructions() {
                let mut defs = Vec::new();
                for &src_reg in insn.source_registers() {
                    for def_addr in self.definitions_of(src_reg, insn.address()) {
                        if !defs.contains(&def_addr.as_u64()) {
                            defs.push(def_addr.as_u64());
                        }
                    }
                }
                chains.insert(insn.address().as_u64(), defs);
            }
        }

        chains
    }

    pub fn cfg(&self) -> &ControlFlowGraph {
        &self.cfg
    }
}

#[derive(Debug, Clone)]
pub struct ValueNumbering {
    values: HashMap<String, u64>,
    next_value: u64,
}

impl ValueNumbering {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            next_value: 0,
        }
    }

    pub fn get_value(&mut self, expr: &str) -> u64 {
        if let Some(&v) = self.values.get(expr) {
            v
        } else {
            let v = self.next_value;
            self.next_value += 1;
            self.values.insert(expr.to_string(), v);
            v
        }
    }

    pub fn has_value(&self, expr: &str) -> bool {
        self.values.contains_key(expr)
    }

    pub fn find_equivalent(&self, value: u64) -> Vec<String> {
        self.values
            .iter()
            .filter(|(_, &v)| v == value)
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.next_value = 0;
    }
}

impl Default for ValueNumbering {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_live_ranges(cfg: &ControlFlowGraph) -> HashMap<u8, Vec<(Address, Address)>> {
    let mut analyzer = DataFlowAnalyzer::new(cfg.clone());
    analyzer.compute_liveness();

    let mut live_ranges: HashMap<u8, Vec<(Address, Address)>> = HashMap::new();

    for block in cfg.blocks() {
        let live_at_entry = analyzer.live_registers_at(block.start());

        for reg in live_at_entry {
            let ranges = live_ranges.entry(reg).or_insert_with(Vec::new);

            let mut in_range = true;
            let mut range_start = block.start();

            for insn in block.instructions() {
                if insn.defines_register(reg) {
                    if in_range {
                        ranges.push((range_start, insn.address()));
                    }
                    range_start = insn.address();
                    in_range = true;
                }

                if !analyzer.is_live_out(reg, insn.address()) {
                    if in_range {
                        ranges.push((range_start, insn.next_address()));
                        in_range = false;
                    }
                }
            }

            if in_range {
                ranges.push((range_start, block.end()));
            }
        }
    }

    live_ranges
}

pub fn find_common_subexpressions(cfg: &ControlFlowGraph) -> Vec<(Address, Address)> {
    let mut cses = Vec::new();
    let mut seen_exprs: HashMap<String, Address> = HashMap::new();

    for block in cfg.blocks() {
        for insn in block.instructions() {
            let expr = format!("{}:{}", insn.mnemonic(), insn.operands_str());

            if let Some(&first_addr) = seen_exprs.get(&expr) {
                cses.push((first_addr, insn.address()));
            } else {
                seen_exprs.insert(expr, insn.address());
            }
        }
    }

    cses
}
