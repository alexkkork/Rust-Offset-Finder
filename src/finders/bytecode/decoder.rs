// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LuauOpcode {
    Nop,
    Break,
    LoadNil,
    LoadB,
    LoadN,
    LoadK,
    Move,
    GetGlobal,
    SetGlobal,
    GetUpval,
    SetUpval,
    CloseUpvals,
    GetImport,
    GetTable,
    SetTable,
    GetTableKS,
    SetTableKS,
    GetTableN,
    SetTableN,
    NewClosure,
    NameCall,
    Call,
    Return,
    Jump,
    JumpBack,
    JumpIf,
    JumpIfNot,
    JumpIfEq,
    JumpIfLe,
    JumpIfLt,
    JumpIfNotEq,
    JumpIfNotLe,
    JumpIfNotLt,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Addk,
    Subk,
    Mulk,
    Divk,
    Modk,
    Powk,
    And,
    Or,
    Andk,
    Ork,
    Concat,
    Not,
    Minus,
    Length,
    NewTable,
    DupTable,
    SetList,
    ForNPrep,
    ForNLoop,
    ForGPrep,
    ForGLoop,
    ForGPrepINext,
    ForGLoopINext,
    ForGPrepNext,
    ForGLoopNext,
    GetVarargs,
    DupClosure,
    PrepVarargs,
    LoadKX,
    JumpX,
    FastCall,
    Coverage,
    Capture,
    JumpIfConstEq,
    JumpIfConstNotEq,
    FastCall1,
    FastCall2,
    FastCall2K,
    ForGPrepInext,
    FastCall3,
    Unknown(u8),
}

impl LuauOpcode {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => LuauOpcode::Nop,
            1 => LuauOpcode::Break,
            2 => LuauOpcode::LoadNil,
            3 => LuauOpcode::LoadB,
            4 => LuauOpcode::LoadN,
            5 => LuauOpcode::LoadK,
            6 => LuauOpcode::Move,
            7 => LuauOpcode::GetGlobal,
            8 => LuauOpcode::SetGlobal,
            9 => LuauOpcode::GetUpval,
            10 => LuauOpcode::SetUpval,
            11 => LuauOpcode::CloseUpvals,
            12 => LuauOpcode::GetImport,
            13 => LuauOpcode::GetTable,
            14 => LuauOpcode::SetTable,
            15 => LuauOpcode::GetTableKS,
            16 => LuauOpcode::SetTableKS,
            17 => LuauOpcode::GetTableN,
            18 => LuauOpcode::SetTableN,
            19 => LuauOpcode::NewClosure,
            20 => LuauOpcode::NameCall,
            21 => LuauOpcode::Call,
            22 => LuauOpcode::Return,
            23 => LuauOpcode::Jump,
            24 => LuauOpcode::JumpBack,
            25 => LuauOpcode::JumpIf,
            26 => LuauOpcode::JumpIfNot,
            27 => LuauOpcode::JumpIfEq,
            28 => LuauOpcode::JumpIfLe,
            29 => LuauOpcode::JumpIfLt,
            30 => LuauOpcode::JumpIfNotEq,
            31 => LuauOpcode::JumpIfNotLe,
            32 => LuauOpcode::JumpIfNotLt,
            33 => LuauOpcode::Add,
            34 => LuauOpcode::Sub,
            35 => LuauOpcode::Mul,
            36 => LuauOpcode::Div,
            37 => LuauOpcode::Mod,
            38 => LuauOpcode::Pow,
            39 => LuauOpcode::Addk,
            40 => LuauOpcode::Subk,
            41 => LuauOpcode::Mulk,
            42 => LuauOpcode::Divk,
            43 => LuauOpcode::Modk,
            44 => LuauOpcode::Powk,
            45 => LuauOpcode::And,
            46 => LuauOpcode::Or,
            47 => LuauOpcode::Andk,
            48 => LuauOpcode::Ork,
            49 => LuauOpcode::Concat,
            50 => LuauOpcode::Not,
            51 => LuauOpcode::Minus,
            52 => LuauOpcode::Length,
            53 => LuauOpcode::NewTable,
            54 => LuauOpcode::DupTable,
            55 => LuauOpcode::SetList,
            56 => LuauOpcode::ForNPrep,
            57 => LuauOpcode::ForNLoop,
            58 => LuauOpcode::ForGPrep,
            59 => LuauOpcode::ForGLoop,
            60 => LuauOpcode::ForGPrepINext,
            61 => LuauOpcode::ForGLoopINext,
            62 => LuauOpcode::ForGPrepNext,
            63 => LuauOpcode::ForGLoopNext,
            64 => LuauOpcode::GetVarargs,
            65 => LuauOpcode::DupClosure,
            66 => LuauOpcode::PrepVarargs,
            67 => LuauOpcode::LoadKX,
            68 => LuauOpcode::JumpX,
            69 => LuauOpcode::FastCall,
            70 => LuauOpcode::Coverage,
            71 => LuauOpcode::Capture,
            72 => LuauOpcode::JumpIfConstEq,
            73 => LuauOpcode::JumpIfConstNotEq,
            74 => LuauOpcode::FastCall1,
            75 => LuauOpcode::FastCall2,
            76 => LuauOpcode::FastCall2K,
            77 => LuauOpcode::ForGPrepInext,
            78 => LuauOpcode::FastCall3,
            _ => LuauOpcode::Unknown(byte),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            LuauOpcode::Nop => 0,
            LuauOpcode::Break => 1,
            LuauOpcode::LoadNil => 2,
            LuauOpcode::LoadB => 3,
            LuauOpcode::LoadN => 4,
            LuauOpcode::LoadK => 5,
            LuauOpcode::Move => 6,
            LuauOpcode::GetGlobal => 7,
            LuauOpcode::SetGlobal => 8,
            LuauOpcode::GetUpval => 9,
            LuauOpcode::SetUpval => 10,
            LuauOpcode::CloseUpvals => 11,
            LuauOpcode::GetImport => 12,
            LuauOpcode::GetTable => 13,
            LuauOpcode::SetTable => 14,
            LuauOpcode::GetTableKS => 15,
            LuauOpcode::SetTableKS => 16,
            LuauOpcode::GetTableN => 17,
            LuauOpcode::SetTableN => 18,
            LuauOpcode::NewClosure => 19,
            LuauOpcode::NameCall => 20,
            LuauOpcode::Call => 21,
            LuauOpcode::Return => 22,
            LuauOpcode::Jump => 23,
            LuauOpcode::JumpBack => 24,
            LuauOpcode::JumpIf => 25,
            LuauOpcode::JumpIfNot => 26,
            LuauOpcode::JumpIfEq => 27,
            LuauOpcode::JumpIfLe => 28,
            LuauOpcode::JumpIfLt => 29,
            LuauOpcode::JumpIfNotEq => 30,
            LuauOpcode::JumpIfNotLe => 31,
            LuauOpcode::JumpIfNotLt => 32,
            LuauOpcode::Add => 33,
            LuauOpcode::Sub => 34,
            LuauOpcode::Mul => 35,
            LuauOpcode::Div => 36,
            LuauOpcode::Mod => 37,
            LuauOpcode::Pow => 38,
            LuauOpcode::Addk => 39,
            LuauOpcode::Subk => 40,
            LuauOpcode::Mulk => 41,
            LuauOpcode::Divk => 42,
            LuauOpcode::Modk => 43,
            LuauOpcode::Powk => 44,
            LuauOpcode::And => 45,
            LuauOpcode::Or => 46,
            LuauOpcode::Andk => 47,
            LuauOpcode::Ork => 48,
            LuauOpcode::Concat => 49,
            LuauOpcode::Not => 50,
            LuauOpcode::Minus => 51,
            LuauOpcode::Length => 52,
            LuauOpcode::NewTable => 53,
            LuauOpcode::DupTable => 54,
            LuauOpcode::SetList => 55,
            LuauOpcode::ForNPrep => 56,
            LuauOpcode::ForNLoop => 57,
            LuauOpcode::ForGPrep => 58,
            LuauOpcode::ForGLoop => 59,
            LuauOpcode::ForGPrepINext => 60,
            LuauOpcode::ForGLoopINext => 61,
            LuauOpcode::ForGPrepNext => 62,
            LuauOpcode::ForGLoopNext => 63,
            LuauOpcode::GetVarargs => 64,
            LuauOpcode::DupClosure => 65,
            LuauOpcode::PrepVarargs => 66,
            LuauOpcode::LoadKX => 67,
            LuauOpcode::JumpX => 68,
            LuauOpcode::FastCall => 69,
            LuauOpcode::Coverage => 70,
            LuauOpcode::Capture => 71,
            LuauOpcode::JumpIfConstEq => 72,
            LuauOpcode::JumpIfConstNotEq => 73,
            LuauOpcode::FastCall1 => 74,
            LuauOpcode::FastCall2 => 75,
            LuauOpcode::FastCall2K => 76,
            LuauOpcode::ForGPrepInext => 77,
            LuauOpcode::FastCall3 => 78,
            LuauOpcode::Unknown(b) => *b,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LuauOpcode::Nop => "NOP",
            LuauOpcode::Break => "BREAK",
            LuauOpcode::LoadNil => "LOADNIL",
            LuauOpcode::LoadB => "LOADB",
            LuauOpcode::LoadN => "LOADN",
            LuauOpcode::LoadK => "LOADK",
            LuauOpcode::Move => "MOVE",
            LuauOpcode::GetGlobal => "GETGLOBAL",
            LuauOpcode::SetGlobal => "SETGLOBAL",
            LuauOpcode::GetUpval => "GETUPVAL",
            LuauOpcode::SetUpval => "SETUPVAL",
            LuauOpcode::CloseUpvals => "CLOSEUPVALS",
            LuauOpcode::GetImport => "GETIMPORT",
            LuauOpcode::GetTable => "GETTABLE",
            LuauOpcode::SetTable => "SETTABLE",
            LuauOpcode::GetTableKS => "GETTABLEKS",
            LuauOpcode::SetTableKS => "SETTABLEKS",
            LuauOpcode::GetTableN => "GETTABLEN",
            LuauOpcode::SetTableN => "SETTABLEN",
            LuauOpcode::NewClosure => "NEWCLOSURE",
            LuauOpcode::NameCall => "NAMECALL",
            LuauOpcode::Call => "CALL",
            LuauOpcode::Return => "RETURN",
            LuauOpcode::Jump => "JUMP",
            LuauOpcode::JumpBack => "JUMPBACK",
            LuauOpcode::JumpIf => "JUMPIF",
            LuauOpcode::JumpIfNot => "JUMPIFNOT",
            LuauOpcode::JumpIfEq => "JUMPIFEQ",
            LuauOpcode::JumpIfLe => "JUMPIFLE",
            LuauOpcode::JumpIfLt => "JUMPIFLT",
            LuauOpcode::JumpIfNotEq => "JUMPIFNOTEQ",
            LuauOpcode::JumpIfNotLe => "JUMPIFNOTLE",
            LuauOpcode::JumpIfNotLt => "JUMPIFNOTLT",
            LuauOpcode::Add => "ADD",
            LuauOpcode::Sub => "SUB",
            LuauOpcode::Mul => "MUL",
            LuauOpcode::Div => "DIV",
            LuauOpcode::Mod => "MOD",
            LuauOpcode::Pow => "POW",
            LuauOpcode::Addk => "ADDK",
            LuauOpcode::Subk => "SUBK",
            LuauOpcode::Mulk => "MULK",
            LuauOpcode::Divk => "DIVK",
            LuauOpcode::Modk => "MODK",
            LuauOpcode::Powk => "POWK",
            LuauOpcode::And => "AND",
            LuauOpcode::Or => "OR",
            LuauOpcode::Andk => "ANDK",
            LuauOpcode::Ork => "ORK",
            LuauOpcode::Concat => "CONCAT",
            LuauOpcode::Not => "NOT",
            LuauOpcode::Minus => "MINUS",
            LuauOpcode::Length => "LENGTH",
            LuauOpcode::NewTable => "NEWTABLE",
            LuauOpcode::DupTable => "DUPTABLE",
            LuauOpcode::SetList => "SETLIST",
            LuauOpcode::ForNPrep => "FORNPREP",
            LuauOpcode::ForNLoop => "FORNLOOP",
            LuauOpcode::ForGPrep => "FORGPREP",
            LuauOpcode::ForGLoop => "FORGLOOP",
            LuauOpcode::ForGPrepINext => "FORGPREPINEXT",
            LuauOpcode::ForGLoopINext => "FORGLOOPINEXT",
            LuauOpcode::ForGPrepNext => "FORGPREPNEXT",
            LuauOpcode::ForGLoopNext => "FORGLOOPNEXT",
            LuauOpcode::GetVarargs => "GETVARARGS",
            LuauOpcode::DupClosure => "DUPCLOSURE",
            LuauOpcode::PrepVarargs => "PREPVARARGS",
            LuauOpcode::LoadKX => "LOADKX",
            LuauOpcode::JumpX => "JUMPX",
            LuauOpcode::FastCall => "FASTCALL",
            LuauOpcode::Coverage => "COVERAGE",
            LuauOpcode::Capture => "CAPTURE",
            LuauOpcode::JumpIfConstEq => "JUMPIFCONSTEQ",
            LuauOpcode::JumpIfConstNotEq => "JUMPIFCONSTNOTEQ",
            LuauOpcode::FastCall1 => "FASTCALL1",
            LuauOpcode::FastCall2 => "FASTCALL2",
            LuauOpcode::FastCall2K => "FASTCALL2K",
            LuauOpcode::ForGPrepInext => "FORGPREPINEXT_",
            LuauOpcode::FastCall3 => "FASTCALL3",
            LuauOpcode::Unknown(_) => "UNKNOWN",
        }
    }

    pub fn operand_count(&self) -> usize {
        match self {
            LuauOpcode::Nop => 0,
            LuauOpcode::Break => 0,
            LuauOpcode::LoadNil => 1,
            LuauOpcode::LoadB => 3,
            LuauOpcode::LoadN => 2,
            LuauOpcode::LoadK => 2,
            LuauOpcode::Move => 2,
            LuauOpcode::GetGlobal => 3,
            LuauOpcode::SetGlobal => 3,
            LuauOpcode::GetUpval => 2,
            LuauOpcode::SetUpval => 2,
            LuauOpcode::CloseUpvals => 1,
            LuauOpcode::GetImport => 3,
            LuauOpcode::GetTable => 3,
            LuauOpcode::SetTable => 3,
            LuauOpcode::GetTableKS => 3,
            LuauOpcode::SetTableKS => 3,
            LuauOpcode::GetTableN => 3,
            LuauOpcode::SetTableN => 3,
            LuauOpcode::NewClosure => 2,
            LuauOpcode::NameCall => 3,
            LuauOpcode::Call => 3,
            LuauOpcode::Return => 2,
            LuauOpcode::Jump => 1,
            LuauOpcode::JumpBack => 1,
            LuauOpcode::JumpIf => 2,
            LuauOpcode::JumpIfNot => 2,
            LuauOpcode::JumpIfEq => 3,
            LuauOpcode::JumpIfLe => 3,
            LuauOpcode::JumpIfLt => 3,
            LuauOpcode::JumpIfNotEq => 3,
            LuauOpcode::JumpIfNotLe => 3,
            LuauOpcode::JumpIfNotLt => 3,
            LuauOpcode::Add => 3,
            LuauOpcode::Sub => 3,
            LuauOpcode::Mul => 3,
            LuauOpcode::Div => 3,
            LuauOpcode::Mod => 3,
            LuauOpcode::Pow => 3,
            LuauOpcode::Addk => 3,
            LuauOpcode::Subk => 3,
            LuauOpcode::Mulk => 3,
            LuauOpcode::Divk => 3,
            LuauOpcode::Modk => 3,
            LuauOpcode::Powk => 3,
            LuauOpcode::And => 3,
            LuauOpcode::Or => 3,
            LuauOpcode::Andk => 3,
            LuauOpcode::Ork => 3,
            LuauOpcode::Concat => 3,
            LuauOpcode::Not => 2,
            LuauOpcode::Minus => 2,
            LuauOpcode::Length => 2,
            LuauOpcode::NewTable => 3,
            LuauOpcode::DupTable => 2,
            LuauOpcode::SetList => 4,
            LuauOpcode::ForNPrep => 2,
            LuauOpcode::ForNLoop => 2,
            LuauOpcode::ForGPrep => 2,
            LuauOpcode::ForGLoop => 3,
            LuauOpcode::ForGPrepINext => 2,
            LuauOpcode::ForGLoopINext => 2,
            LuauOpcode::ForGPrepNext => 2,
            LuauOpcode::ForGLoopNext => 2,
            LuauOpcode::GetVarargs => 2,
            LuauOpcode::DupClosure => 2,
            LuauOpcode::PrepVarargs => 1,
            LuauOpcode::LoadKX => 1,
            LuauOpcode::JumpX => 1,
            LuauOpcode::FastCall => 2,
            LuauOpcode::Coverage => 1,
            LuauOpcode::Capture => 2,
            LuauOpcode::JumpIfConstEq => 3,
            LuauOpcode::JumpIfConstNotEq => 3,
            LuauOpcode::FastCall1 => 3,
            LuauOpcode::FastCall2 => 4,
            LuauOpcode::FastCall2K => 4,
            LuauOpcode::ForGPrepInext => 2,
            LuauOpcode::FastCall3 => 5,
            LuauOpcode::Unknown(_) => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LuauInstruction {
    pub opcode: LuauOpcode,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub aux: Option<u32>,
    pub raw: u32,
}

impl LuauInstruction {
    pub fn decode(raw: u32) -> Self {
        let opcode_byte = (raw & 0xFF) as u8;
        let a = ((raw >> 8) & 0xFF) as u8;
        let b = ((raw >> 16) & 0xFF) as u8;
        let c = ((raw >> 24) & 0xFF) as u8;

        Self {
            opcode: LuauOpcode::from_byte(opcode_byte),
            a,
            b,
            c,
            aux: None,
            raw,
        }
    }

    pub fn with_aux(mut self, aux: u32) -> Self {
        self.aux = Some(aux);
        self
    }

    pub fn sbx(&self) -> i32 {
        let sbx = ((self.raw >> 16) & 0xFFFF) as i32;
        sbx - 0x7FFF
    }

    pub fn bx(&self) -> u32 {
        (self.raw >> 16) & 0xFFFF
    }
}

pub struct BytecodeDecoder {
    reader: Arc<dyn MemoryReader>,
}

impl BytecodeDecoder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn decode_function(&self, addr: Address, instruction_count: usize) -> Vec<LuauInstruction> {
        let mut instructions = Vec::with_capacity(instruction_count);

        let byte_count = instruction_count * 4;
        if let Ok(bytes) = self.reader.read_bytes(addr, byte_count) {
            let mut i = 0;
            while i < bytes.len() - 3 {
                let raw = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                let mut insn = LuauInstruction::decode(raw);

                let needs_aux = matches!(
                    insn.opcode,
                    LuauOpcode::GetGlobal
                        | LuauOpcode::SetGlobal
                        | LuauOpcode::GetImport
                        | LuauOpcode::GetTableKS
                        | LuauOpcode::SetTableKS
                        | LuauOpcode::NameCall
                        | LuauOpcode::LoadK
                        | LuauOpcode::DupClosure
                        | LuauOpcode::JumpIfConstEq
                        | LuauOpcode::JumpIfConstNotEq
                        | LuauOpcode::FastCall2
                        | LuauOpcode::FastCall2K
                        | LuauOpcode::ForGLoop
                        | LuauOpcode::LoadKX
                );

                if needs_aux && i + 7 < bytes.len() {
                    let aux = u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]);
                    insn = insn.with_aux(aux);
                    i += 8;
                } else {
                    i += 4;
                }

                instructions.push(insn);
            }
        }

        instructions
    }
}
