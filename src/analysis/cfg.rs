// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disassembler::{Disassembler, DisassembledInstruction};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    entry: Address,
    blocks: HashMap<u64, CfgBlock>,
    edges: Vec<CfgEdge>,
    exit_blocks: Vec<u64>,
}

impl ControlFlowGraph {
    pub fn new(entry: Address) -> Self {
        Self {
            entry,
            blocks: HashMap::new(),
            edges: Vec::new(),
            exit_blocks: Vec::new(),
        }
    }

    pub fn build(reader: Arc<dyn MemoryReader>, entry: Address, max_size: usize) -> Result<Self, MemoryError> {
        let disasm = Disassembler::new(reader);
        let mut cfg = Self::new(entry);
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(entry);

        while let Some(block_start) = queue.pop_front() {
            if visited.contains(&block_start.as_u64()) {
                continue;
            }

            if block_start.as_u64() < entry.as_u64() || 
               block_start.as_u64() > entry.as_u64() + max_size as u64 {
                continue;
            }

            visited.insert(block_start.as_u64());

            let mut instructions = Vec::new();
            let mut current = block_start;
            let mut successors = Vec::new();
            let mut is_exit = false;

            loop {
                if current.as_u64() > entry.as_u64() + max_size as u64 {
                    break;
                }

                let instr = disasm.disassemble(current)?;

                if disasm.is_return_instruction(&instr) {
                    instructions.push(instr);
                    is_exit = true;
                    break;
                }

                if disasm.is_branch_instruction(&instr) {
                    instructions.push(instr.clone());

                    if let Some(target) = disasm.get_branch_target(&instr) {
                        successors.push(target);
                        queue.push_back(target);
                    }

                    if instr.mnemonic != "B" && instr.mnemonic != "BR" {
                        let fallthrough = current + 4;
                        successors.push(fallthrough);
                        queue.push_back(fallthrough);
                    }

                    break;
                }

                if disasm.is_call_instruction(&instr) {
                    instructions.push(instr.clone());
                    current = current + 4;

                    if visited.contains(&current.as_u64()) {
                        successors.push(current);
                        break;
                    }

                    continue;
                }

                instructions.push(instr);
                current = current + 4;

                if visited.contains(&current.as_u64()) {
                    successors.push(current);
                    break;
                }
            }

            let block_end = if instructions.is_empty() {
                block_start
            } else {
                instructions.last().unwrap().address
            };

            let block = CfgBlock {
                start: block_start,
                end: block_end,
                instructions,
                predecessors: Vec::new(),
                successors: successors.clone(),
            };

            cfg.blocks.insert(block_start.as_u64(), block);

            if is_exit {
                cfg.exit_blocks.push(block_start.as_u64());
            }

            for succ in successors {
                cfg.edges.push(CfgEdge {
                    from: block_start,
                    to: succ,
                    edge_type: EdgeType::Flow,
                });
            }
        }

        cfg.compute_predecessors();
        Ok(cfg)
    }

    fn compute_predecessors(&mut self) {
        for edge in &self.edges {
            if let Some(block) = self.blocks.get_mut(&edge.to.as_u64()) {
                block.predecessors.push(edge.from);
            }
        }
    }

    pub fn entry(&self) -> Address {
        self.entry
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_block(&self, addr: Address) -> Option<&CfgBlock> {
        self.blocks.get(&addr.as_u64())
    }

    pub fn blocks(&self) -> impl Iterator<Item = &CfgBlock> {
        self.blocks.values()
    }

    pub fn edges(&self) -> &[CfgEdge] {
        &self.edges
    }

    pub fn exit_blocks(&self) -> &[u64] {
        &self.exit_blocks
    }

    pub fn is_exit_block(&self, addr: Address) -> bool {
        self.exit_blocks.contains(&addr.as_u64())
    }

    pub fn add_block(&mut self, block: CfgBlock) {
        self.blocks.insert(block.start.as_u64(), block);
    }

    pub fn add_edge(&mut self, from_id: u64, to_id: u64) {
        self.edges.push(CfgEdge {
            from: Address::new(from_id),
            to: Address::new(to_id),
            edge_type: EdgeType::Flow,
        });
    }

    pub fn predecessors(&self, addr: Address) -> Vec<Address> {
        if let Some(block) = self.blocks.get(&addr.as_u64()) {
            block.predecessors.clone()
        } else {
            Vec::new()
        }
    }

    pub fn dominators(&self) -> HashMap<u64, HashSet<u64>> {
        let mut dom: HashMap<u64, HashSet<u64>> = HashMap::new();

        for &addr in self.blocks.keys() {
            let mut all: HashSet<u64> = self.blocks.keys().cloned().collect();
            dom.insert(addr, all);
        }

        if let Some(entry_dom) = dom.get_mut(&self.entry.as_u64()) {
            entry_dom.clear();
            entry_dom.insert(self.entry.as_u64());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for (&addr, block) in &self.blocks {
                if addr == self.entry.as_u64() {
                    continue;
                }

                let mut new_dom: Option<HashSet<u64>> = None;

                for pred in &block.predecessors {
                    if let Some(pred_dom) = dom.get(&pred.as_u64()) {
                        match &mut new_dom {
                            None => new_dom = Some(pred_dom.clone()),
                            Some(d) => *d = d.intersection(pred_dom).cloned().collect(),
                        }
                    }
                }

                let mut result = new_dom.unwrap_or_default();
                result.insert(addr);

                if let Some(current) = dom.get(&addr) {
                    if result != *current {
                        changed = true;
                        dom.insert(addr, result);
                    }
                }
            }
        }

        dom
    }

    pub fn post_order(&self) -> Vec<u64> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        fn dfs(cfg: &ControlFlowGraph, addr: u64, visited: &mut HashSet<u64>, result: &mut Vec<u64>) {
            if visited.contains(&addr) {
                return;
            }
            visited.insert(addr);

            if let Some(block) = cfg.blocks.get(&addr) {
                for succ in &block.successors {
                    dfs(cfg, succ.as_u64(), visited, result);
                }
            }

            result.push(addr);
        }

        dfs(self, self.entry.as_u64(), &mut visited, &mut result);
        result
    }

    pub fn reverse_post_order(&self) -> Vec<u64> {
        let mut order = self.post_order();
        order.reverse();
        order
    }
}

#[derive(Debug, Clone)]
pub struct CfgBlock {
    pub start: Address,
    pub end: Address,
    pub instructions: Vec<DisassembledInstruction>,
    pub predecessors: Vec<Address>,
    pub successors: Vec<Address>,
}

impl CfgBlock {
    pub fn id(&self) -> u64 {
        self.start.as_u64()
    }

    pub fn instructions(&self) -> &[DisassembledInstruction] {
        &self.instructions
    }

    pub fn start_address(&self) -> Address {
        self.start
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_entry(&self) -> bool {
        self.predecessors.is_empty()
    }

    pub fn is_exit(&self) -> bool {
        self.successors.is_empty()
    }

    pub fn has_call(&self) -> bool {
        self.instructions.iter().any(|i| i.mnemonic == "BL" || i.mnemonic == "BLR")
    }
}

#[derive(Debug, Clone)]
pub struct CfgEdge {
    pub from: Address,
    pub to: Address,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Flow,
    ConditionalTrue,
    ConditionalFalse,
    Call,
    Return,
    Jump,
}
