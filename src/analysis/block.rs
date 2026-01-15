// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::Instruction;
use std::fmt;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    start: Address,
    end: Address,
    instructions: Vec<Instruction>,
    predecessors: Vec<Address>,
    successors: Vec<Address>,
    is_entry: bool,
    is_exit: bool,
    dominator: Option<Address>,
    loop_header: Option<Address>,
}

impl BasicBlock {
    pub fn new(start: Address, end: Address, instructions: Vec<Instruction>) -> Self {
        Self {
            start,
            end,
            instructions,
            predecessors: Vec::new(),
            successors: Vec::new(),
            is_entry: false,
            is_exit: false,
            dominator: None,
            loop_header: None,
        }
    }

    pub fn start(&self) -> Address {
        self.start
    }

    pub fn end(&self) -> Address {
        self.end
    }

    pub fn size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn first_instruction(&self) -> Option<&Instruction> {
        self.instructions.first()
    }

    pub fn last_instruction(&self) -> Option<&Instruction> {
        self.instructions.last()
    }

    pub fn instruction_at(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn contains(&self, addr: Address) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn predecessors(&self) -> &[Address] {
        &self.predecessors
    }

    pub fn add_predecessor(&mut self, pred: Address) {
        if !self.predecessors.contains(&pred) {
            self.predecessors.push(pred);
        }
    }

    pub fn remove_predecessor(&mut self, pred: Address) {
        self.predecessors.retain(|&p| p != pred);
    }

    pub fn successors(&self) -> &[Address] {
        &self.successors
    }

    pub fn add_successor(&mut self, succ: Address) {
        if !self.successors.contains(&succ) {
            self.successors.push(succ);
        }
    }

    pub fn remove_successor(&mut self, succ: Address) {
        self.successors.retain(|&s| s != succ);
    }

    pub fn predecessor_count(&self) -> usize {
        self.predecessors.len()
    }

    pub fn successor_count(&self) -> usize {
        self.successors.len()
    }

    pub fn is_entry(&self) -> bool {
        self.is_entry
    }

    pub fn set_entry(&mut self, is_entry: bool) {
        self.is_entry = is_entry;
    }

    pub fn is_exit(&self) -> bool {
        self.is_exit
    }

    pub fn set_exit(&mut self, is_exit: bool) {
        self.is_exit = is_exit;
    }

    pub fn dominator(&self) -> Option<Address> {
        self.dominator
    }

    pub fn set_dominator(&mut self, dom: Address) {
        self.dominator = Some(dom);
    }

    pub fn clear_dominator(&mut self) {
        self.dominator = None;
    }

    pub fn loop_header(&self) -> Option<Address> {
        self.loop_header
    }

    pub fn set_loop_header(&mut self, header: Address) {
        self.loop_header = Some(header);
    }

    pub fn is_loop_header(&self) -> bool {
        self.loop_header.map(|h| h == self.start).unwrap_or(false)
    }

    pub fn is_in_loop(&self) -> bool {
        self.loop_header.is_some()
    }

    pub fn has_fall_through(&self) -> bool {
        if let Some(last) = self.last_instruction() {
            !last.is_unconditional_branch() && !last.is_return()
        } else {
            true
        }
    }

    pub fn has_conditional_branch(&self) -> bool {
        if let Some(last) = self.last_instruction() {
            last.is_conditional_branch()
        } else {
            false
        }
    }

    pub fn has_unconditional_branch(&self) -> bool {
        if let Some(last) = self.last_instruction() {
            last.is_unconditional_branch()
        } else {
            false
        }
    }

    pub fn has_call(&self) -> bool {
        self.instructions.iter().any(|i| i.is_call())
    }

    pub fn has_return(&self) -> bool {
        if let Some(last) = self.last_instruction() {
            last.is_return()
        } else {
            false
        }
    }

    pub fn get_calls(&self) -> Vec<Address> {
        self.instructions
            .iter()
            .filter(|i| i.is_call())
            .filter_map(|i| i.branch_target())
            .collect()
    }

    pub fn get_branch_targets(&self) -> Vec<Address> {
        if let Some(last) = self.last_instruction() {
            last.branch_targets()
        } else {
            Vec::new()
        }
    }

    pub fn split_at(&self, addr: Address) -> Option<(BasicBlock, BasicBlock)> {
        if !self.contains(addr) || addr == self.start {
            return None;
        }

        let split_index = self.instructions
            .iter()
            .position(|i| i.address() == addr)?;

        let first_instructions = self.instructions[..split_index].to_vec();
        let second_instructions = self.instructions[split_index..].to_vec();

        let first = BasicBlock::new(self.start, addr, first_instructions);
        let second = BasicBlock::new(addr, self.end, second_instructions);

        Some((first, second))
    }

    pub fn merge(&self, other: &BasicBlock) -> Option<BasicBlock> {
        if self.end != other.start {
            return None;
        }

        let mut instructions = self.instructions.clone();
        instructions.extend(other.instructions.clone());

        let mut merged = BasicBlock::new(self.start, other.end, instructions);
        merged.predecessors = self.predecessors.clone();
        merged.successors = other.successors.clone();

        Some(merged)
    }

    pub fn analyze_data_flow(&self) -> BlockDataFlow {
        let mut defined: Vec<u8> = Vec::new();
        let mut used: Vec<u8> = Vec::new();
        let mut killed: Vec<u8> = Vec::new();

        for insn in &self.instructions {
            for &src in insn.source_registers() {
                if !defined.contains(&src) {
                    used.push(src);
                }
            }

            if let Some(dst) = insn.destination_register() {
                if !defined.contains(&dst) {
                    defined.push(dst);
                }
                killed.push(dst);
            }
        }

        BlockDataFlow {
            defined,
            used,
            killed,
        }
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Block {:016x} - {:016x} ({} instructions)",
            self.start.as_u64(), self.end.as_u64(), self.instructions.len())?;

        if !self.predecessors.is_empty() {
            write!(f, "  Predecessors: ")?;
            for (i, pred) in self.predecessors.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{:016x}", pred.as_u64())?;
            }
            writeln!(f)?;
        }

        if !self.successors.is_empty() {
            write!(f, "  Successors: ")?;
            for (i, succ) in self.successors.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{:016x}", succ.as_u64())?;
            }
            writeln!(f)?;
        }

        for insn in &self.instructions {
            writeln!(f, "    {}", insn)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BlockDataFlow {
    pub defined: Vec<u8>,
    pub used: Vec<u8>,
    pub killed: Vec<u8>,
}

impl BlockDataFlow {
    pub fn new() -> Self {
        Self {
            defined: Vec::new(),
            used: Vec::new(),
            killed: Vec::new(),
        }
    }

    pub fn is_live_in(&self, reg: u8) -> bool {
        self.used.contains(&reg)
    }

    pub fn is_live_out(&self, reg: u8) -> bool {
        self.defined.contains(&reg) && !self.killed.contains(&reg)
    }

    pub fn is_killed(&self, reg: u8) -> bool {
        self.killed.contains(&reg)
    }
}

impl Default for BlockDataFlow {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_block_graph(blocks: &mut [BasicBlock]) {
    let block_starts: std::collections::HashSet<u64> = blocks
        .iter()
        .map(|b| b.start().as_u64())
        .collect();

    for i in 0..blocks.len() {
        let block_end = blocks[i].end();
        let targets = blocks[i].get_branch_targets();
        let has_fall_through = blocks[i].has_fall_through();

        let mut successors = Vec::new();

        for target in targets {
            if block_starts.contains(&target.as_u64()) {
                successors.push(target);
            }
        }

        if has_fall_through && block_starts.contains(&block_end.as_u64()) {
            successors.push(block_end);
        }

        blocks[i].successors = successors;
    }

    let successor_map: std::collections::HashMap<u64, Vec<Address>> = blocks
        .iter()
        .map(|b| (b.start().as_u64(), b.successors().to_vec()))
        .collect();

    for block in blocks.iter_mut() {
        let block_start = block.start();
        for (&pred_start, succs) in &successor_map {
            if succs.iter().any(|s| s.as_u64() == block_start.as_u64()) {
                block.add_predecessor(Address::new(pred_start));
            }
        }
    }

    if let Some(first) = blocks.first_mut() {
        first.set_entry(true);
    }

    for block in blocks.iter_mut() {
        if block.has_return() || block.successors().is_empty() {
            block.set_exit(true);
        }
    }
}
