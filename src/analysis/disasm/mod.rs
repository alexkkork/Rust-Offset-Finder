// Wed Jan 15 2026 - Alex

pub mod arm64;
pub mod engine;
pub mod formatter;
pub mod iterator;
pub mod cache;

pub use engine::DisassemblyEngine;
pub use formatter::InstructionFormatter;
pub use iterator::InstructionIterator;
pub use cache::DisassemblyCache;

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;

pub struct DisassemblyContext {
    reader: Arc<dyn MemoryReader>,
    engine: DisassemblyEngine,
    cache: DisassemblyCache,
    config: DisassemblyConfig,
}

impl DisassemblyContext {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            engine: DisassemblyEngine::new(reader),
            cache: DisassemblyCache::new(10000),
            config: DisassemblyConfig::default(),
        }
    }

    pub fn disassemble_at(&mut self, addr: Address) -> Result<DecodedInstruction, MemoryError> {
        if let Some(cached) = self.cache.get(addr) {
            return Ok(cached);
        }

        let instruction = self.engine.decode(addr)?;
        self.cache.insert(addr, instruction.clone());
        Ok(instruction)
    }

    pub fn disassemble_range(&mut self, start: Address, end: Address) -> Result<Vec<DecodedInstruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = start;

        while current < end {
            let instr = self.disassemble_at(current)?;
            let size = instr.size;
            instructions.push(instr);
            current = current + size as u64;
        }

        Ok(instructions)
    }

    pub fn disassemble_function(&mut self, entry: Address) -> Result<Vec<DecodedInstruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = entry;
        let max_instructions = self.config.max_function_instructions;

        for _ in 0..max_instructions {
            let instr = self.disassemble_at(current)?;
            let is_ret = instr.is_return();
            let size = instr.size;
            instructions.push(instr);

            if is_ret {
                break;
            }

            current = current + size as u64;
        }

        Ok(instructions)
    }

    pub fn iter_from(&mut self, start: Address) -> InstructionIterator {
        InstructionIterator::new(self.reader.clone(), start, self.config.max_function_instructions)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    pub address: Address,
    pub bytes: Vec<u8>,
    pub size: usize,
    pub mnemonic: String,
    pub operands: Vec<Operand>,
    pub operand_str: String,
    pub raw: u32,
    pub category: InstructionCategory,
}

impl DecodedInstruction {
    pub fn is_branch(&self) -> bool {
        matches!(self.category, 
            InstructionCategory::Branch | 
            InstructionCategory::ConditionalBranch |
            InstructionCategory::Call
        )
    }

    pub fn is_call(&self) -> bool {
        matches!(self.category, InstructionCategory::Call)
    }

    pub fn is_return(&self) -> bool {
        matches!(self.category, InstructionCategory::Return)
    }

    pub fn is_conditional(&self) -> bool {
        matches!(self.category, InstructionCategory::ConditionalBranch)
    }

    pub fn is_load(&self) -> bool {
        matches!(self.category, InstructionCategory::Load)
    }

    pub fn is_store(&self) -> bool {
        matches!(self.category, InstructionCategory::Store)
    }

    pub fn get_branch_target(&self) -> Option<Address> {
        if !self.is_branch() && !self.is_call() {
            return None;
        }

        for op in &self.operands {
            if let Operand::Immediate(imm) = op {
                return Some(Address::new(*imm as u64));
            }
            if let Operand::Address(addr) = op {
                return Some(*addr);
            }
        }

        None
    }

    pub fn format(&self) -> String {
        format!("{:016X}  {}  {}", 
            self.address.as_u64(), 
            self.mnemonic, 
            self.operand_str
        )
    }
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(u8),
    Immediate(i64),
    Address(Address),
    Memory { base: u8, offset: i64, index: Option<u8>, scale: u8 },
    Condition(u8),
    ShiftedReg { reg: u8, shift_type: ShiftType, amount: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
    Arithmetic,
    Logic,
    Move,
    Load,
    Store,
    Branch,
    ConditionalBranch,
    Call,
    Return,
    Compare,
    System,
    Simd,
    Crypto,
    Unknown,
}

impl InstructionCategory {
    pub fn name(&self) -> &'static str {
        match self {
            InstructionCategory::Arithmetic => "Arithmetic",
            InstructionCategory::Logic => "Logic",
            InstructionCategory::Move => "Move",
            InstructionCategory::Load => "Load",
            InstructionCategory::Store => "Store",
            InstructionCategory::Branch => "Branch",
            InstructionCategory::ConditionalBranch => "Conditional Branch",
            InstructionCategory::Call => "Call",
            InstructionCategory::Return => "Return",
            InstructionCategory::Compare => "Compare",
            InstructionCategory::System => "System",
            InstructionCategory::Simd => "SIMD",
            InstructionCategory::Crypto => "Crypto",
            InstructionCategory::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisassemblyConfig {
    pub max_function_instructions: usize,
    pub follow_calls: bool,
    pub resolve_symbols: bool,
    pub show_bytes: bool,
}

impl Default for DisassemblyConfig {
    fn default() -> Self {
        Self {
            max_function_instructions: 10000,
            follow_calls: false,
            resolve_symbols: true,
            show_bytes: true,
        }
    }
}
