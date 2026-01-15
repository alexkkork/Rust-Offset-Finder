// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::arm64::{Arm64Instruction, Arm64Decoder};
use crate::analysis::{BasicBlock, Instruction};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Disassembler {
    reader: Arc<dyn MemoryReader>,
    decoder: Arm64Decoder,
    cache: HashMap<u64, Arm64Instruction>,
    max_cache_size: usize,
}

impl Disassembler {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            decoder: Arm64Decoder::new(),
            cache: HashMap::new(),
            max_cache_size: 100000,
        }
    }

    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    pub fn disassemble_at(&mut self, address: Address) -> Result<Arm64Instruction, MemoryError> {
        let addr_u64 = address.as_u64();

        if let Some(cached) = self.cache.get(&addr_u64) {
            return Ok(cached.clone());
        }

        let raw = self.reader.read_u32(address)?;
        let insn = self.decoder.decode(address, raw);

        if self.cache.len() >= self.max_cache_size {
            self.cache.clear();
        }
        self.cache.insert(addr_u64, insn.clone());

        Ok(insn)
    }

    pub fn disassemble_range(&mut self, start: Address, end: Address) -> Result<Vec<Arm64Instruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = start;

        while current < end {
            let insn = self.disassemble_at(current)?;
            instructions.push(insn);
            current = current + 4;
        }

        Ok(instructions)
    }

    pub fn disassemble_function(&mut self, start: Address) -> Result<Vec<Arm64Instruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = start;
        let mut visited = HashSet::new();
        let max_size = 0x10000;

        while (current.as_u64() - start.as_u64()) < max_size {
            if visited.contains(&current.as_u64()) {
                break;
            }
            visited.insert(current.as_u64());

            let insn = match self.disassemble_at(current) {
                Ok(i) => i,
                Err(_) => break,
            };

            let is_return = insn.is_return();
            let is_unconditional = insn.is_unconditional_branch();

            instructions.push(insn);

            if is_return {
                break;
            }

            if is_unconditional {
                break;
            }

            current = current + 4;
        }

        Ok(instructions)
    }

    pub fn disassemble_basic_block(&mut self, start: Address) -> Result<BasicBlock, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = start;
        let max_size = 0x1000;

        while (current.as_u64() - start.as_u64()) < max_size {
            let insn = match self.disassemble_at(current) {
                Ok(i) => i,
                Err(_) => break,
            };

            let is_terminator = insn.is_branch() || insn.is_return();
            instructions.push(Instruction::from_arm64(&insn));

            if is_terminator {
                break;
            }

            current = current + 4;
        }

        let end = if instructions.is_empty() {
            start
        } else {
            start + (instructions.len() * 4) as u64
        };

        Ok(BasicBlock::new(start, end, instructions))
    }

    pub fn analyze_control_flow(&mut self, start: Address) -> Result<Vec<BasicBlock>, MemoryError> {
        let mut blocks = Vec::new();
        let mut work_queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut block_starts = HashSet::new();

        work_queue.push_back(start);
        block_starts.insert(start.as_u64());

        while let Some(block_start) = work_queue.pop_front() {
            if visited.contains(&block_start.as_u64()) {
                continue;
            }
            visited.insert(block_start.as_u64());

            let block = self.disassemble_basic_block(block_start)?;

            if let Some(last_insn) = block.instructions().last() {
                if last_insn.is_branch() {
                    for target in last_insn.branch_targets() {
                        if !block_starts.contains(&target.as_u64()) {
                            block_starts.insert(target.as_u64());
                            work_queue.push_back(target);
                        }
                    }

                    if last_insn.is_conditional_branch() {
                        let fallthrough = block.end();
                        if !block_starts.contains(&fallthrough.as_u64()) {
                            block_starts.insert(fallthrough.as_u64());
                            work_queue.push_back(fallthrough);
                        }
                    }
                }
            }

            blocks.push(block);
        }

        blocks.sort_by_key(|b| b.start().as_u64());
        Ok(blocks)
    }

    pub fn find_function_calls(&mut self, start: Address, end: Address) -> Result<Vec<(Address, Address)>, MemoryError> {
        let mut calls = Vec::new();
        let mut current = start;

        while current < end {
            if let Ok(insn) = self.disassemble_at(current) {
                if insn.is_call() {
                    if let Some(target) = insn.branch_target() {
                        calls.push((current, target));
                    }
                }
            }
            current = current + 4;
        }

        Ok(calls)
    }

    pub fn find_xrefs_to(&mut self, target: Address, search_start: Address, search_end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut xrefs = Vec::new();
        let mut current = search_start;

        while current < search_end {
            if let Ok(insn) = self.disassemble_at(current) {
                if insn.is_branch() || insn.is_call() {
                    if let Some(branch_target) = insn.branch_target() {
                        if branch_target.as_u64() == target.as_u64() {
                            xrefs.push(current);
                        }
                    }
                }

                for operand in &insn.operands {
                    if let crate::analysis::arm64::OperandType::PCRelative(offset) = operand.op_type {
                        let resolved = Address::new((current.as_u64() as i64 + offset as i64) as u64);
                        if resolved.as_u64() == target.as_u64() {
                            xrefs.push(current);
                        }
                    }
                }
            }
            current = current + 4;
        }

        Ok(xrefs)
    }

    pub fn find_string_references(&mut self, string_addr: Address, search_start: Address, search_end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut refs = Vec::new();
        let mut current = search_start;

        while current < search_end {
            if let Ok(insn) = self.disassemble_at(current) {
                for operand in &insn.operands {
                    if let crate::analysis::arm64::OperandType::PCRelative(offset) = operand.op_type {
                        let resolved = Address::new((current.as_u64() as i64 + offset as i64) as u64);
                        if resolved.as_u64() == string_addr.as_u64() {
                            refs.push(current);
                        }
                    }
                    if let crate::analysis::arm64::OperandType::Immediate(imm) = operand.op_type {
                        if imm as u64 == string_addr.as_u64() {
                            refs.push(current);
                        }
                    }
                }
            }
            current = current + 4;
        }

        Ok(refs)
    }

    pub fn estimate_function_size(&mut self, start: Address) -> Result<u64, MemoryError> {
        let instructions = self.disassemble_function(start)?;
        Ok((instructions.len() * 4) as u64)
    }

    pub fn is_function_prologue(&mut self, address: Address) -> Result<bool, MemoryError> {
        let insn = self.disassemble_at(address)?;

        if let crate::analysis::arm64::Opcode::STP = insn.opcode {
            for operand in &insn.operands {
                if let crate::analysis::arm64::OperandType::Register(reg) = operand.op_type {
                    if reg.index() == 29 || reg.index() == 30 {
                        return Ok(true);
                    }
                }
            }
        }

        if let crate::analysis::arm64::Opcode::SUB = insn.opcode {
            if insn.operands.len() >= 2 {
                if let crate::analysis::arm64::OperandType::Register(reg) = insn.operands[0].op_type {
                    if reg.index() == 31 {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn is_function_epilogue(&mut self, address: Address) -> Result<bool, MemoryError> {
        let insn = self.disassemble_at(address)?;

        if insn.is_return() {
            return Ok(true);
        }

        if let crate::analysis::arm64::Opcode::LDP = insn.opcode {
            for operand in &insn.operands {
                if let crate::analysis::arm64::OperandType::Register(reg) = operand.op_type {
                    if reg.index() == 29 || reg.index() == 30 {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn find_function_boundaries(&mut self, start: Address, max_search: u64) -> Result<(Address, Address), MemoryError> {
        let mut func_start = start;
        let mut func_end = start;

        let mut current = start;
        while current.as_u64() > start.as_u64().saturating_sub(max_search) {
            if self.is_function_prologue(current)? {
                func_start = current;
                break;
            }
            current = current - 4;
        }

        current = start;
        while current.as_u64() < start.as_u64() + max_search {
            if let Ok(insn) = self.disassemble_at(current) {
                if insn.is_return() {
                    func_end = current + 4;
                    break;
                }
            } else {
                break;
            }
            current = current + 4;
        }

        Ok((func_start, func_end))
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

pub fn disassemble_bytes(bytes: &[u8], base_address: Address) -> Vec<Arm64Instruction> {
    let decoder = Arm64Decoder::new();
    let mut instructions = Vec::new();

    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        let raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let addr = base_address + (i * 4) as u64;
        let insn = decoder.decode(addr, raw);
        instructions.push(insn);
    }

    instructions
}

pub fn format_instruction(insn: &Arm64Instruction) -> String {
    format!("{}", insn)
}

pub fn format_disassembly(instructions: &[Arm64Instruction]) -> String {
    let mut output = String::new();
    for insn in instructions {
        output.push_str(&format!("{}\n", insn));
    }
    output
}
