// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::disassembler::DisassembledInstruction;
use crate::analysis::cfg::ControlFlowGraph;
use std::collections::{HashMap, HashSet};

pub struct DataFlowAnalyzer {
    cfg: ControlFlowGraph,
}

impl DataFlowAnalyzer {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        Self { cfg }
    }

    pub fn compute_reaching_definitions(&self) -> ReachingDefinitions {
        let mut rd = ReachingDefinitions::new();

        let mut gen: HashMap<u64, HashSet<Definition>> = HashMap::new();
        let mut kill: HashMap<u64, HashSet<String>> = HashMap::new();

        for block in self.cfg.blocks() {
            let block_id = block.id();
            let mut block_gen = HashSet::new();
            let mut block_kill = HashSet::new();

            for instr in block.instructions() {
                if let Some(def_reg) = self.get_defined_register(instr) {
                    block_kill.insert(def_reg.clone());

                    block_gen.insert(Definition {
                        register: def_reg,
                        address: instr.address,
                        block_id,
                    });
                }
            }

            gen.insert(block_id, block_gen);
            kill.insert(block_id, block_kill);
        }

        let mut in_sets: HashMap<u64, HashSet<Definition>> = HashMap::new();
        let mut out_sets: HashMap<u64, HashSet<Definition>> = HashMap::new();

        for block in self.cfg.blocks() {
            in_sets.insert(block.id(), HashSet::new());
            out_sets.insert(block.id(), HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in self.cfg.blocks() {
                let block_id = block.id();

                let mut new_in = HashSet::new();
                for pred_addr in self.cfg.predecessors(block.start_address()) {
                    let pred_id = pred_addr.as_u64();
                    if let Some(pred_out) = out_sets.get(&pred_id) {
                        new_in.extend(pred_out.iter().cloned());
                    }
                }

                let block_gen = gen.get(&block_id).cloned().unwrap_or_default();
                let block_kill = kill.get(&block_id).cloned().unwrap_or_default();

                let mut new_out: HashSet<Definition> = new_in.iter()
                    .filter(|def| !block_kill.contains(&def.register))
                    .cloned()
                    .collect();
                new_out.extend(block_gen);

                if new_out != *out_sets.get(&block_id).unwrap() {
                    changed = true;
                }

                in_sets.insert(block_id, new_in);
                out_sets.insert(block_id, new_out);
            }
        }

        rd.in_sets = in_sets;
        rd.out_sets = out_sets;

        rd
    }

    pub fn compute_live_variables(&self) -> LiveVariables {
        let mut lv = LiveVariables::new();

        let mut use_sets: HashMap<u64, HashSet<String>> = HashMap::new();
        let mut def_sets: HashMap<u64, HashSet<String>> = HashMap::new();

        for block in self.cfg.blocks() {
            let block_id = block.id();
            let mut block_use = HashSet::new();
            let mut block_def = HashSet::new();

            for instr in block.instructions() {
                let used_regs = self.get_used_registers(instr);
                for reg in used_regs {
                    if !block_def.contains(&reg) {
                        block_use.insert(reg);
                    }
                }

                if let Some(def_reg) = self.get_defined_register(instr) {
                    block_def.insert(def_reg);
                }
            }

            use_sets.insert(block_id, block_use);
            def_sets.insert(block_id, block_def);
        }

        let mut in_sets: HashMap<u64, HashSet<String>> = HashMap::new();
        let mut out_sets: HashMap<u64, HashSet<String>> = HashMap::new();

        for block in self.cfg.blocks() {
            in_sets.insert(block.id(), HashSet::new());
            out_sets.insert(block.id(), HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            let blocks: Vec<_> = self.cfg.blocks().collect();
            for block in blocks.iter().rev() {
                let block_id = block.id();

                let mut new_out = HashSet::new();
                for succ_addr in &block.successors {
                    let succ_id = succ_addr.as_u64();
                    if let Some(succ_in) = in_sets.get(&succ_id) {
                        new_out.extend(succ_in.iter().cloned());
                    }
                }

                let block_use = use_sets.get(&block_id).cloned().unwrap_or_default();
                let block_def = def_sets.get(&block_id).cloned().unwrap_or_default();

                let mut new_in: HashSet<String> = new_out.iter()
                    .filter(|var| !block_def.contains(*var))
                    .cloned()
                    .collect();
                new_in.extend(block_use);

                if new_in != *in_sets.get(&block_id).unwrap() {
                    changed = true;
                }

                in_sets.insert(block_id, new_in);
                out_sets.insert(block_id, new_out);
            }
        }

        lv.in_sets = in_sets;
        lv.out_sets = out_sets;

        lv
    }

    pub fn compute_def_use_chains(&self) -> DefUseChains {
        let mut chains = DefUseChains::new();

        let rd = self.compute_reaching_definitions();

        for block in self.cfg.blocks() {
            let block_id = block.id();
            let mut current_defs = rd.in_sets.get(&block_id)
                .cloned()
                .unwrap_or_default();

            for instr in block.instructions() {
                let used_regs = self.get_used_registers(instr);
                for reg in used_regs {
                    let reaching: Vec<Address> = current_defs.iter()
                        .filter(|def| def.register == reg)
                        .map(|def| def.address)
                        .collect();

                    for def_addr in reaching {
                        chains.add_use(def_addr, instr.address, reg.clone());
                    }
                }

                if let Some(def_reg) = self.get_defined_register(instr) {
                    current_defs.retain(|def| def.register != def_reg);

                    current_defs.insert(Definition {
                        register: def_reg.clone(),
                        address: instr.address,
                        block_id,
                    });

                    chains.add_definition(instr.address, def_reg);
                }
            }
        }

        chains
    }

    pub fn compute_available_expressions(&self) -> AvailableExpressions {
        let mut ae = AvailableExpressions::new();

        let mut gen: HashMap<u64, HashSet<Expression>> = HashMap::new();
        let mut kill: HashMap<u64, HashSet<Expression>> = HashMap::new();

        for block in self.cfg.blocks() {
            let block_id = block.id();
            let mut block_gen = HashSet::new();
            let mut block_kill = HashSet::new();

            for instr in block.instructions() {
                if let Some(expr) = self.extract_expression(instr) {
                    if let Some(def_reg) = self.get_defined_register(instr) {
                        block_kill.extend(
                            block_gen.iter()
                                .filter(|e: &&Expression| e.uses_register(&def_reg))
                                .cloned()
                        );
                    }
                    block_gen.insert(expr);
                }

                if let Some(def_reg) = self.get_defined_register(instr) {
                    block_kill.extend(
                        ae.all_expressions.iter()
                            .filter(|e| e.uses_register(&def_reg))
                            .cloned()
                    );
                }
            }

            gen.insert(block_id, block_gen);
            kill.insert(block_id, block_kill);
        }

        let mut in_sets: HashMap<u64, HashSet<Expression>> = HashMap::new();
        let mut out_sets: HashMap<u64, HashSet<Expression>> = HashMap::new();

        let all_exprs: HashSet<Expression> = gen.values().flatten().cloned().collect();

        for block in self.cfg.blocks() {
            let block_id = block.id();
            if self.cfg.predecessors(block.start_address()).is_empty() {
                in_sets.insert(block_id, HashSet::new());
            } else {
                in_sets.insert(block_id, all_exprs.clone());
            }
            out_sets.insert(block_id, HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for block in self.cfg.blocks() {
                let block_id = block.id();
                let preds = self.cfg.predecessors(block.start_address());

                let new_in = if preds.is_empty() {
                    HashSet::new()
                } else {
                    let mut result = all_exprs.clone();
                    for pred_addr in preds {
                        let pred_id = pred_addr.as_u64();
                        if let Some(pred_out) = out_sets.get(&pred_id) {
                            result = result.intersection(pred_out).cloned().collect();
                        }
                    }
                    result
                };

                let block_gen = gen.get(&block_id).cloned().unwrap_or_default();
                let block_kill = kill.get(&block_id).cloned().unwrap_or_default();

                let mut new_out: HashSet<Expression> = new_in.iter()
                    .filter(|expr| !block_kill.contains(*expr))
                    .cloned()
                    .collect();
                new_out.extend(block_gen);

                if new_out != *out_sets.get(&block_id).unwrap() {
                    changed = true;
                }

                in_sets.insert(block_id, new_in);
                out_sets.insert(block_id, new_out);
            }
        }

        ae.in_sets = in_sets;
        ae.out_sets = out_sets;

        ae
    }

    fn get_defined_register(&self, instr: &DisassembledInstruction) -> Option<String> {
        let parts: Vec<&str> = instr.op_str.split(',').collect();
        if !parts.is_empty() {
            let dest = parts[0].trim();
            if dest.starts_with('X') || dest.starts_with('W') {
                return Some(dest.to_string());
            }
        }
        None
    }

    fn get_used_registers(&self, instr: &DisassembledInstruction) -> Vec<String> {
        let mut regs = Vec::new();
        let parts: Vec<&str> = instr.op_str.split(',').collect();

        for part in parts.iter().skip(1) {
            let operand = part.trim();
            if operand.starts_with('X') || operand.starts_with('W') {
                let reg = operand.split(|c: char| !c.is_alphanumeric())
                    .next()
                    .unwrap_or(operand);
                regs.push(reg.to_string());
            }

            if operand.contains('[') {
                for word in operand.split(|c: char| !c.is_alphanumeric()) {
                    if word.starts_with('X') || word.starts_with('W') {
                        regs.push(word.to_string());
                    }
                }
            }
        }

        regs
    }

    fn extract_expression(&self, instr: &DisassembledInstruction) -> Option<Expression> {
        match instr.mnemonic.as_str() {
            "ADD" | "SUB" | "MUL" | "SDIV" | "UDIV" | "AND" | "ORR" | "EOR" | "LSL" | "LSR" | "ASR" => {
                let parts: Vec<&str> = instr.op_str.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 3 {
                    Some(Expression {
                        operation: instr.mnemonic.clone(),
                        operands: vec![parts[1].to_string(), parts[2].to_string()],
                        address: instr.address,
                    })
                } else {
                    None
                }
            }
            "LDR" => {
                let parts: Vec<&str> = instr.op_str.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    Some(Expression {
                        operation: "LOAD".to_string(),
                        operands: vec![parts[1].to_string()],
                        address: instr.address,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReachingDefinitions {
    pub in_sets: HashMap<u64, HashSet<Definition>>,
    pub out_sets: HashMap<u64, HashSet<Definition>>,
}

impl ReachingDefinitions {
    pub fn new() -> Self {
        Self {
            in_sets: HashMap::new(),
            out_sets: HashMap::new(),
        }
    }

    pub fn reaching_at(&self, block_id: u64) -> Option<&HashSet<Definition>> {
        self.in_sets.get(&block_id)
    }
}

impl Default for ReachingDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    pub register: String,
    pub address: Address,
    pub block_id: u64,
}

#[derive(Debug, Clone)]
pub struct LiveVariables {
    pub in_sets: HashMap<u64, HashSet<String>>,
    pub out_sets: HashMap<u64, HashSet<String>>,
}

impl LiveVariables {
    pub fn new() -> Self {
        Self {
            in_sets: HashMap::new(),
            out_sets: HashMap::new(),
        }
    }

    pub fn live_in(&self, block_id: u64) -> Option<&HashSet<String>> {
        self.in_sets.get(&block_id)
    }

    pub fn live_out(&self, block_id: u64) -> Option<&HashSet<String>> {
        self.out_sets.get(&block_id)
    }
}

impl Default for LiveVariables {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DefUseChains {
    pub definitions: HashMap<Address, String>,
    pub uses: HashMap<Address, Vec<Use>>,
}

impl DefUseChains {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            uses: HashMap::new(),
        }
    }

    pub fn add_definition(&mut self, addr: Address, register: String) {
        self.definitions.insert(addr, register);
    }

    pub fn add_use(&mut self, def_addr: Address, use_addr: Address, register: String) {
        self.uses.entry(def_addr).or_default().push(Use {
            address: use_addr,
            register,
        });
    }

    pub fn uses_of(&self, def_addr: Address) -> Option<&Vec<Use>> {
        self.uses.get(&def_addr)
    }
}

impl Default for DefUseChains {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Use {
    pub address: Address,
    pub register: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expression {
    pub operation: String,
    pub operands: Vec<String>,
    pub address: Address,
}

impl Expression {
    pub fn uses_register(&self, reg: &str) -> bool {
        self.operands.iter().any(|op| op.contains(reg))
    }
}

#[derive(Debug, Clone)]
pub struct AvailableExpressions {
    pub in_sets: HashMap<u64, HashSet<Expression>>,
    pub out_sets: HashMap<u64, HashSet<Expression>>,
    pub all_expressions: HashSet<Expression>,
}

impl AvailableExpressions {
    pub fn new() -> Self {
        Self {
            in_sets: HashMap::new(),
            out_sets: HashMap::new(),
            all_expressions: HashSet::new(),
        }
    }

    pub fn available_at(&self, block_id: u64) -> Option<&HashSet<Expression>> {
        self.in_sets.get(&block_id)
    }
}

impl Default for AvailableExpressions {
    fn default() -> Self {
        Self::new()
    }
}
