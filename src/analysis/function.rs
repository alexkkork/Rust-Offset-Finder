// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError, MemoryRegion};
use crate::analysis::disassembler::{Disassembler, DisassembledInstruction};
use crate::analysis::block::{BasicBlock, BlockDataFlow};
use crate::analysis::cfg::ControlFlowGraph;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct FunctionAnalyzer {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
}

impl FunctionAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>, disassembler: Arc<Disassembler>) -> Self {
        Self { reader, disassembler }
    }

    pub fn analyze(&self, entry_point: Address) -> Result<AnalyzedFunction, MemoryError> {
        let mut function = AnalyzedFunction::new(entry_point);

        let instructions = self.disassembler.disassemble_function(entry_point, 0x10000)?;

        if instructions.is_empty() {
            return Ok(function);
        }

        let blocks = self.identify_basic_blocks(&instructions);

        for block in &blocks {
            function.add_block(block.clone());
        }

        let cfg = self.build_cfg(&blocks);
        function.cfg = Some(cfg);

        self.analyze_prologue(&instructions, &mut function);
        self.analyze_epilogue(&instructions, &mut function);

        self.analyze_stack_frame(&instructions, &mut function);

        self.analyze_function_calls(&instructions, &mut function);

        self.analyze_data_references(&instructions, &mut function);

        self.analyze_register_usage(&instructions, &mut function);

        function.size = self.calculate_function_size(&instructions);

        Ok(function)
    }

    fn identify_basic_blocks(&self, instructions: &[DisassembledInstruction]) -> Vec<BasicBlock> {
        let mut block_starts: HashSet<u64> = HashSet::new();
        let mut block_ends: HashSet<u64> = HashSet::new();

        if let Some(first) = instructions.first() {
            block_starts.insert(first.address.as_u64());
        }

        for instr in instructions {
            if instr.is_branch() || instr.is_call() {
                block_ends.insert(instr.address.as_u64());

                if let Some(target) = instr.branch_target() {
                    block_starts.insert(target);
                }

                let next_addr = instr.address.as_u64() + instr.size as u64;
                block_starts.insert(next_addr);
            }

            if instr.is_return() {
                block_ends.insert(instr.address.as_u64());
            }
        }

        let mut sorted_starts: Vec<u64> = block_starts.into_iter().collect();
        sorted_starts.sort();

        let mut blocks = Vec::new();
        let mut current_block_idx = 0;

        for (idx, &start) in sorted_starts.iter().enumerate() {
            let end = if idx + 1 < sorted_starts.len() {
                sorted_starts[idx + 1]
            } else {
                instructions.last()
                    .map(|i| i.address.as_u64() + i.size as u64)
                    .unwrap_or(start)
            };

            let block_instructions: Vec<DisassembledInstruction> = instructions.iter()
                .filter(|i| {
                    let addr = i.address.as_u64();
                    addr >= start && addr < end
                })
                .cloned()
                .collect();

            if !block_instructions.is_empty() {
                let mut block = BasicBlock::new(current_block_idx, Address::new(start));
                for instr in block_instructions {
                    block.add_instruction(instr);
                }
                blocks.push(block);
                current_block_idx += 1;
            }
        }

        blocks
    }

    fn build_cfg(&self, blocks: &[BasicBlock]) -> ControlFlowGraph {
        let mut cfg = ControlFlowGraph::new();

        for block in blocks {
            cfg.add_block(block.clone());
        }

        for block in blocks {
            if let Some(last_instr) = block.instructions().last() {
                if last_instr.is_branch() {
                    if let Some(target) = last_instr.branch_target() {
                        if let Some(target_block) = blocks.iter()
                            .find(|b| b.start_address().as_u64() == target)
                        {
                            cfg.add_edge(block.id(), target_block.id());
                        }
                    }

                    if last_instr.is_conditional_branch() {
                        let fall_through = last_instr.address.as_u64() + last_instr.size as u64;
                        if let Some(fall_through_block) = blocks.iter()
                            .find(|b| b.start_address().as_u64() == fall_through)
                        {
                            cfg.add_edge(block.id(), fall_through_block.id());
                        }
                    }
                } else if !last_instr.is_return() {
                    let fall_through = last_instr.address.as_u64() + last_instr.size as u64;
                    if let Some(fall_through_block) = blocks.iter()
                        .find(|b| b.start_address().as_u64() == fall_through)
                    {
                        cfg.add_edge(block.id(), fall_through_block.id());
                    }
                }
            }
        }

        cfg
    }

    fn analyze_prologue(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        for (idx, instr) in instructions.iter().take(10).enumerate() {
            if instr.mnemonic.starts_with("STP") && instr.op_str.contains("X29") && instr.op_str.contains("X30") {
                function.has_frame_pointer = true;
                function.prologue_size = (idx + 1) * 4;
            }

            if instr.mnemonic == "SUB" && instr.op_str.contains("SP") {
                if let Some(stack_size) = self.extract_immediate(&instr.op_str) {
                    function.stack_size = stack_size as usize;
                }
            }

            if instr.mnemonic.starts_with("STP") || instr.mnemonic.starts_with("STR") {
                if instr.op_str.contains("X19") || instr.op_str.contains("X20") ||
                   instr.op_str.contains("X21") || instr.op_str.contains("X22") ||
                   instr.op_str.contains("X23") || instr.op_str.contains("X24") ||
                   instr.op_str.contains("X25") || instr.op_str.contains("X26") ||
                   instr.op_str.contains("X27") || instr.op_str.contains("X28")
                {
                    for i in 19..=28 {
                        let reg = format!("X{}", i);
                        if instr.op_str.contains(&reg) {
                            function.saved_registers.push(reg);
                        }
                    }
                }
            }
        }
    }

    fn analyze_epilogue(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        let len = instructions.len();
        if len < 2 {
            return;
        }

        for (idx, instr) in instructions.iter().rev().take(10).enumerate() {
            if instr.mnemonic == "RET" {
                function.epilogue_size = 4;
            }

            if instr.mnemonic.starts_with("LDP") && instr.op_str.contains("X29") && instr.op_str.contains("X30") {
                function.epilogue_size += 4;
            }

            if instr.mnemonic == "ADD" && instr.op_str.contains("SP") {
                function.epilogue_size += 4;
            }
        }
    }

    fn analyze_stack_frame(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        let mut stack_accesses: HashMap<i64, StackAccess> = HashMap::new();

        for instr in instructions {
            if instr.mnemonic.starts_with("STR") || instr.mnemonic.starts_with("STP") {
                if let Some(offset) = self.extract_stack_offset(&instr.op_str) {
                    let access = stack_accesses.entry(offset).or_insert_with(|| {
                        StackAccess {
                            offset,
                            size: self.get_access_size(&instr.mnemonic),
                            read_count: 0,
                            write_count: 0,
                        }
                    });
                    access.write_count += 1;
                }
            }

            if instr.mnemonic.starts_with("LDR") || instr.mnemonic.starts_with("LDP") {
                if let Some(offset) = self.extract_stack_offset(&instr.op_str) {
                    let access = stack_accesses.entry(offset).or_insert_with(|| {
                        StackAccess {
                            offset,
                            size: self.get_access_size(&instr.mnemonic),
                            read_count: 0,
                            write_count: 0,
                        }
                    });
                    access.read_count += 1;
                }
            }
        }

        function.stack_accesses = stack_accesses.into_values().collect();
    }

    fn analyze_function_calls(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        for instr in instructions {
            if instr.is_call() {
                if let Some(target) = instr.branch_target() {
                    function.called_functions.push(FunctionCall {
                        call_site: instr.address,
                        target: Address::new(target),
                        is_direct: true,
                    });
                } else {
                    function.called_functions.push(FunctionCall {
                        call_site: instr.address,
                        target: Address::new(0),
                        is_direct: false,
                    });
                }
            }
        }
    }

    fn analyze_data_references(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        for instr in instructions {
            if instr.mnemonic == "ADRP" || instr.mnemonic == "ADR" {
                if let Some(target) = self.extract_immediate(&instr.op_str) {
                    function.data_references.push(DataReference {
                        instruction_address: instr.address,
                        target_address: Address::new(target as u64),
                        reference_type: DataReferenceType::Address,
                    });
                }
            }

            if instr.mnemonic == "LDR" && instr.op_str.contains("[PC") {
                if let Some(offset) = self.extract_immediate(&instr.op_str) {
                    let target = instr.address.as_u64().wrapping_add(offset as u64);
                    function.data_references.push(DataReference {
                        instruction_address: instr.address,
                        target_address: Address::new(target),
                        reference_type: DataReferenceType::PcRelative,
                    });
                }
            }
        }
    }

    fn analyze_register_usage(&self, instructions: &[DisassembledInstruction], function: &mut AnalyzedFunction) {
        let mut reg_defs: HashMap<String, Vec<Address>> = HashMap::new();
        let mut reg_uses: HashMap<String, Vec<Address>> = HashMap::new();

        for instr in instructions {
            let parts: Vec<&str> = instr.op_str.split(',').collect();

            if !parts.is_empty() {
                let dest = parts[0].trim();
                if dest.starts_with('X') || dest.starts_with('W') {
                    reg_defs.entry(dest.to_string())
                        .or_default()
                        .push(instr.address);
                }
            }

            for part in parts.iter().skip(1) {
                let operand = part.trim();
                if operand.starts_with('X') || operand.starts_with('W') {
                    let reg = operand.split(|c: char| !c.is_alphanumeric())
                        .next()
                        .unwrap_or(operand);
                    reg_uses.entry(reg.to_string())
                        .or_default()
                        .push(instr.address);
                }
            }
        }

        function.register_definitions = reg_defs;
        function.register_uses = reg_uses;
    }

    fn calculate_function_size(&self, instructions: &[DisassembledInstruction]) -> usize {
        if instructions.is_empty() {
            return 0;
        }

        let first = instructions.first().unwrap().address.as_u64();
        let last = instructions.last().unwrap();
        let end = last.address.as_u64() + last.size as u64;

        (end - first) as usize
    }

    fn extract_immediate(&self, op_str: &str) -> Option<i64> {
        for part in op_str.split(|c: char| c == ',' || c == ' ' || c == '[' || c == ']') {
            let trimmed = part.trim().trim_start_matches('#');
            if let Ok(val) = trimmed.parse::<i64>() {
                return Some(val);
            }
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                if let Ok(val) = i64::from_str_radix(&trimmed[2..], 16) {
                    return Some(val);
                }
            }
        }
        None
    }

    fn extract_stack_offset(&self, op_str: &str) -> Option<i64> {
        if op_str.contains("SP") || op_str.contains("X29") {
            self.extract_immediate(op_str)
        } else {
            None
        }
    }

    fn get_access_size(&self, mnemonic: &str) -> usize {
        if mnemonic.contains('B') {
            1
        } else if mnemonic.contains('H') {
            2
        } else if mnemonic.contains('W') || mnemonic.starts_with("STR W") || mnemonic.starts_with("LDR W") {
            4
        } else if mnemonic.starts_with("STP") || mnemonic.starts_with("LDP") {
            16
        } else {
            8
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzedFunction {
    pub entry_point: Address,
    pub size: usize,
    pub blocks: Vec<BasicBlock>,
    pub cfg: Option<ControlFlowGraph>,
    pub has_frame_pointer: bool,
    pub stack_size: usize,
    pub prologue_size: usize,
    pub epilogue_size: usize,
    pub saved_registers: Vec<String>,
    pub called_functions: Vec<FunctionCall>,
    pub data_references: Vec<DataReference>,
    pub stack_accesses: Vec<StackAccess>,
    pub register_definitions: HashMap<String, Vec<Address>>,
    pub register_uses: HashMap<String, Vec<Address>>,
}

impl AnalyzedFunction {
    pub fn new(entry_point: Address) -> Self {
        Self {
            entry_point,
            size: 0,
            blocks: Vec::new(),
            cfg: None,
            has_frame_pointer: false,
            stack_size: 0,
            prologue_size: 0,
            epilogue_size: 0,
            saved_registers: Vec::new(),
            called_functions: Vec::new(),
            data_references: Vec::new(),
            stack_accesses: Vec::new(),
            register_definitions: HashMap::new(),
            register_uses: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.push(block);
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn instruction_count(&self) -> usize {
        self.blocks.iter()
            .map(|b| b.instruction_count())
            .sum()
    }

    pub fn end_address(&self) -> Address {
        Address::new(self.entry_point.as_u64() + self.size as u64)
    }

    pub fn contains(&self, addr: Address) -> bool {
        let addr_val = addr.as_u64();
        addr_val >= self.entry_point.as_u64() && addr_val < self.end_address().as_u64()
    }

    pub fn is_leaf(&self) -> bool {
        self.called_functions.is_empty()
    }

    pub fn callee_count(&self) -> usize {
        self.called_functions.iter()
            .filter(|c| c.is_direct)
            .map(|c| c.target)
            .collect::<HashSet<_>>()
            .len()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub call_site: Address,
    pub target: Address,
    pub is_direct: bool,
}

#[derive(Debug, Clone)]
pub struct DataReference {
    pub instruction_address: Address,
    pub target_address: Address,
    pub reference_type: DataReferenceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataReferenceType {
    Address,
    PcRelative,
    GotEntry,
    TlsAccess,
}

#[derive(Debug, Clone)]
pub struct StackAccess {
    pub offset: i64,
    pub size: usize,
    pub read_count: u32,
    pub write_count: u32,
}

impl StackAccess {
    pub fn is_read_only(&self) -> bool {
        self.read_count > 0 && self.write_count == 0
    }

    pub fn is_write_only(&self) -> bool {
        self.write_count > 0 && self.read_count == 0
    }

    pub fn total_accesses(&self) -> u32 {
        self.read_count + self.write_count
    }
}
