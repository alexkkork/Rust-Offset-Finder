// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::opcode::{LuauOpcode, OpcodeInfo};
use std::sync::Arc;
use std::collections::HashMap;

pub struct LuauBytecode {
    instructions: Vec<BytecodeInstruction>,
    constants: Vec<BytecodeConstant>,
    protos: Vec<ProtoInfo>,
    upvalues: Vec<UpvalueInfo>,
    debug_info: Option<DebugData>,
}

impl LuauBytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            protos: Vec::new(),
            upvalues: Vec::new(),
            debug_info: None,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, BytecodeError> {
        let reader = BytecodeReader::new(data);
        reader.read_bytecode()
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn constant_count(&self) -> usize {
        self.constants.len()
    }

    pub fn proto_count(&self) -> usize {
        self.protos.len()
    }

    pub fn get_instruction(&self, index: usize) -> Option<&BytecodeInstruction> {
        self.instructions.get(index)
    }

    pub fn get_constant(&self, index: usize) -> Option<&BytecodeConstant> {
        self.constants.get(index)
    }

    pub fn instructions(&self) -> &[BytecodeInstruction] {
        &self.instructions
    }

    pub fn constants(&self) -> &[BytecodeConstant] {
        &self.constants
    }

    pub fn protos(&self) -> &[ProtoInfo] {
        &self.protos
    }

    pub fn has_debug_info(&self) -> bool {
        self.debug_info.is_some()
    }

    pub fn disassemble(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("; Instructions: {}\n", self.instructions.len()));
        output.push_str(&format!("; Constants: {}\n", self.constants.len()));
        output.push_str(&format!("; Protos: {}\n", self.protos.len()));
        output.push_str("\n");

        for (idx, instr) in self.instructions.iter().enumerate() {
            let opcode_info = OpcodeInfo::from_opcode(instr.opcode);
            output.push_str(&format!("{:04}: {} {}\n", idx, opcode_info.name, instr.operands_string()));
        }

        output
    }
}

impl Default for LuauBytecode {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BytecodeReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> BytecodeReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_bytecode(&self) -> Result<LuauBytecode, BytecodeError> {
        let mut bytecode = LuauBytecode::new();

        if self.data.len() < 8 {
            return Err(BytecodeError::InvalidFormat("Data too short".to_string()));
        }

        let version = self.data[0];
        if version != 3 && version != 4 && version != 5 {
            return Err(BytecodeError::UnsupportedVersion(version));
        }

        Ok(bytecode)
    }

    fn read_u8(&mut self) -> Result<u8, BytecodeError> {
        if self.offset >= self.data.len() {
            return Err(BytecodeError::UnexpectedEof);
        }
        let value = self.data[self.offset];
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, BytecodeError> {
        if self.offset + 2 > self.data.len() {
            return Err(BytecodeError::UnexpectedEof);
        }
        let value = u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]);
        self.offset += 2;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, BytecodeError> {
        if self.offset + 4 > self.data.len() {
            return Err(BytecodeError::UnexpectedEof);
        }
        let value = u32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(value)
    }

    fn read_varint(&mut self) -> Result<u32, BytecodeError> {
        let mut value: u32 = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;
            value |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                break;
            }

            shift += 7;
            if shift >= 35 {
                return Err(BytecodeError::InvalidFormat("Varint too large".to_string()));
            }
        }

        Ok(value)
    }

    fn read_string(&mut self) -> Result<String, BytecodeError> {
        let len = self.read_varint()? as usize;

        if len == 0 {
            return Ok(String::new());
        }

        if self.offset + len > self.data.len() {
            return Err(BytecodeError::UnexpectedEof);
        }

        let bytes = &self.data[self.offset..self.offset + len];
        self.offset += len;

        String::from_utf8(bytes.to_vec())
            .map_err(|_| BytecodeError::InvalidString)
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeInstruction {
    pub opcode: LuauOpcode,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: i16,
    pub aux: Option<u32>,
    pub raw: u32,
}

impl BytecodeInstruction {
    pub fn from_u32(raw: u32) -> Self {
        let opcode = LuauOpcode::from_u8((raw & 0xFF) as u8);
        let a = ((raw >> 8) & 0xFF) as u8;
        let b = ((raw >> 16) & 0xFF) as u8;
        let c = ((raw >> 24) & 0xFF) as u8;
        let d = ((raw >> 16) & 0xFFFF) as i16;

        Self {
            opcode,
            a,
            b,
            c,
            d,
            aux: None,
            raw,
        }
    }

    pub fn with_aux(mut self, aux: u32) -> Self {
        self.aux = Some(aux);
        self
    }

    pub fn operands_string(&self) -> String {
        let info = OpcodeInfo::from_opcode(self.opcode);

        match info.format {
            OpcodeFormat::None => String::new(),
            OpcodeFormat::A => format!("R{}", self.a),
            OpcodeFormat::AB => format!("R{}, R{}", self.a, self.b),
            OpcodeFormat::ABC => format!("R{}, R{}, R{}", self.a, self.b, self.c),
            OpcodeFormat::AD => format!("R{}, {}", self.a, self.d),
            OpcodeFormat::AsBx => format!("R{}, {}", self.a, self.d - 0x8000),
            OpcodeFormat::ABx => format!("R{}, K{}", self.a, ((self.raw >> 16) & 0xFFFF)),
            OpcodeFormat::Ax => format!("{}", self.raw >> 8),
        }
    }

    pub fn is_jump(&self) -> bool {
        matches!(self.opcode,
            LuauOpcode::Jump |
            LuauOpcode::JumpBack |
            LuauOpcode::JumpIf |
            LuauOpcode::JumpIfNot |
            LuauOpcode::JumpIfEq |
            LuauOpcode::JumpIfLe |
            LuauOpcode::JumpIfLt |
            LuauOpcode::JumpIfNotEq |
            LuauOpcode::JumpIfNotLe |
            LuauOpcode::JumpIfNotLt
        )
    }

    pub fn is_call(&self) -> bool {
        matches!(self.opcode,
            LuauOpcode::Call |
            LuauOpcode::TailCall
        )
    }

    pub fn is_return(&self) -> bool {
        matches!(self.opcode, LuauOpcode::Return)
    }

    pub fn jump_target(&self, current_pc: usize) -> Option<usize> {
        if self.is_jump() {
            Some((current_pc as i32 + self.d as i32 + 1) as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum BytecodeConstant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Import(u32),
    Table(Vec<u32>),
    Closure(u32),
}

impl BytecodeConstant {
    pub fn type_name(&self) -> &'static str {
        match self {
            BytecodeConstant::Nil => "nil",
            BytecodeConstant::Boolean(_) => "boolean",
            BytecodeConstant::Number(_) => "number",
            BytecodeConstant::String(_) => "string",
            BytecodeConstant::Import(_) => "import",
            BytecodeConstant::Table(_) => "table",
            BytecodeConstant::Closure(_) => "closure",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtoInfo {
    pub maxstacksize: u8,
    pub numparams: u8,
    pub nups: u8,
    pub is_vararg: bool,
    pub linedefined: u32,
    pub sizecode: u32,
    pub sizek: u32,
    pub sizep: u32,
    pub sizelineinfo: u32,
}

impl ProtoInfo {
    pub fn new() -> Self {
        Self {
            maxstacksize: 0,
            numparams: 0,
            nups: 0,
            is_vararg: false,
            linedefined: 0,
            sizecode: 0,
            sizek: 0,
            sizep: 0,
            sizelineinfo: 0,
        }
    }
}

impl Default for ProtoInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct UpvalueInfo {
    pub name: Option<String>,
    pub instack: bool,
    pub idx: u8,
}

#[derive(Debug, Clone)]
pub struct DebugData {
    pub source: String,
    pub line_info: Vec<i32>,
    pub local_vars: Vec<LocalVarInfo>,
    pub upvalue_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LocalVarInfo {
    pub name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum OpcodeFormat {
    None,
    A,
    AB,
    ABC,
    AD,
    AsBx,
    ABx,
    Ax,
}

#[derive(Debug)]
pub enum BytecodeError {
    InvalidFormat(String),
    UnsupportedVersion(u8),
    UnexpectedEof,
    InvalidOpcode(u8),
    InvalidString,
    InvalidConstant,
}

impl std::fmt::Display for BytecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BytecodeError::InvalidFormat(msg) => write!(f, "Invalid bytecode format: {}", msg),
            BytecodeError::UnsupportedVersion(v) => write!(f, "Unsupported bytecode version: {}", v),
            BytecodeError::UnexpectedEof => write!(f, "Unexpected end of bytecode"),
            BytecodeError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02X}", op),
            BytecodeError::InvalidString => write!(f, "Invalid UTF-8 string in bytecode"),
            BytecodeError::InvalidConstant => write!(f, "Invalid constant in bytecode"),
        }
    }
}

impl std::error::Error for BytecodeError {}

pub struct BytecodeAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl BytecodeAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze_proto_at(&self, addr: Address) -> Result<ProtoAnalysis, MemoryError> {
        let bytes = self.reader.read_bytes(addr, 0x100)?;

        let analysis = ProtoAnalysis {
            address: addr,
            code_address: Address::new(0),
            constant_address: Address::new(0),
            instruction_count: 0,
            constant_count: 0,
            upvalue_count: 0,
            parameter_count: 0,
            max_stack_size: 0,
            is_vararg: false,
        };

        Ok(analysis)
    }

    pub fn find_protos_in_range(&self, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut protos = Vec::new();

        Ok(protos)
    }

    pub fn decode_instruction_at(&self, addr: Address) -> Result<BytecodeInstruction, MemoryError> {
        let raw = self.reader.read_u32(addr)?;
        let instr = BytecodeInstruction::from_u32(raw);
        Ok(instr)
    }

    pub fn decode_instructions_at(&self, addr: Address, count: usize) -> Result<Vec<BytecodeInstruction>, MemoryError> {
        let mut instructions = Vec::with_capacity(count);

        for i in 0..count {
            let instr_addr = addr + (i * 4) as u64;
            let instr = self.decode_instruction_at(instr_addr)?;
            instructions.push(instr);
        }

        Ok(instructions)
    }
}

#[derive(Debug, Clone)]
pub struct ProtoAnalysis {
    pub address: Address,
    pub code_address: Address,
    pub constant_address: Address,
    pub instruction_count: usize,
    pub constant_count: usize,
    pub upvalue_count: usize,
    pub parameter_count: u8,
    pub max_stack_size: u8,
    pub is_vararg: bool,
}
