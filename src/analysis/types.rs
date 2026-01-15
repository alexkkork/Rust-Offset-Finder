// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::{ControlFlowGraph, Instruction};
use std::collections::HashMap;
use std::sync::Arc;

pub struct TypeAnalyzer {
    reader: Arc<dyn MemoryReader>,
    type_info: HashMap<u64, InferredType>,
    register_types: HashMap<(u64, u8), InferredType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferredType {
    Unknown,
    Integer(IntegerType),
    Float(FloatType),
    Pointer(Box<InferredType>),
    Array(Box<InferredType>, usize),
    Struct(Vec<(String, InferredType)>),
    Function(FunctionSignature),
    String,
    Boolean,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub return_type: Box<InferredType>,
    pub param_types: Vec<InferredType>,
    pub is_variadic: bool,
}

impl TypeAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            type_info: HashMap::new(),
            register_types: HashMap::new(),
        }
    }

    pub fn analyze_value(&mut self, addr: Address) -> Result<InferredType, MemoryError> {
        if let Some(cached) = self.type_info.get(&addr.as_u64()) {
            return Ok(cached.clone());
        }

        let value = self.reader.read_u64(addr)?;
        let ty = self.infer_type_from_value(value);
        self.type_info.insert(addr.as_u64(), ty.clone());

        Ok(ty)
    }

    pub fn analyze_instruction(&mut self, insn: &Instruction) {
        let addr = insn.address().as_u64();

        if insn.is_load() {
            if let Some(dest) = insn.destination_register() {
                if let Some(mem) = insn.memory_operand() {
                    let ty = match mem.size {
                        1 => InferredType::Integer(IntegerType::U8),
                        2 => InferredType::Integer(IntegerType::U16),
                        4 => InferredType::Integer(IntegerType::U32),
                        8 => InferredType::Integer(IntegerType::U64),
                        _ => InferredType::Unknown,
                    };
                    self.register_types.insert((addr, dest), ty);
                }
            }
        }

        if insn.is_float() {
            if let Some(dest) = insn.destination_register() {
                self.register_types.insert((addr, dest), InferredType::Float(FloatType::F64));
            }
        }

        if insn.is_compare() {
            if let Some(dest) = insn.destination_register() {
                self.register_types.insert((addr, dest), InferredType::Boolean);
            }
        }

        if insn.is_call() {
            if let Some(dest) = insn.destination_register() {
                self.register_types.insert((addr, dest), InferredType::Unknown);
            }
        }
    }

    pub fn analyze_function(&mut self, cfg: &ControlFlowGraph) {
        for block in cfg.blocks() {
            for insn in block.instructions() {
                self.analyze_instruction(insn);
            }
        }
    }

    fn infer_type_from_value(&self, value: u64) -> InferredType {
        if value == 0 {
            return InferredType::Pointer(Box::new(InferredType::Unknown));
        }

        if value >= 0x100000000 && value < 0x800000000000 {
            return InferredType::Pointer(Box::new(InferredType::Unknown));
        }

        if value <= 0xFF {
            return InferredType::Integer(IntegerType::U8);
        }

        if value <= 0xFFFF {
            return InferredType::Integer(IntegerType::U16);
        }

        if value <= 0xFFFFFFFF {
            return InferredType::Integer(IntegerType::U32);
        }

        InferredType::Integer(IntegerType::U64)
    }

    pub fn get_type_at(&self, addr: Address) -> Option<&InferredType> {
        self.type_info.get(&addr.as_u64())
    }

    pub fn get_register_type(&self, addr: Address, reg: u8) -> Option<&InferredType> {
        self.register_types.get(&(addr.as_u64(), reg))
    }

    pub fn set_type_at(&mut self, addr: Address, ty: InferredType) {
        self.type_info.insert(addr.as_u64(), ty);
    }

    pub fn propagate_types(&mut self, cfg: &ControlFlowGraph) {
        let mut changed = true;
        let mut iterations = 0;
        let max_iterations = 100;

        while changed && iterations < max_iterations {
            changed = false;
            iterations += 1;

            for block in cfg.blocks() {
                for insn in block.instructions() {
                    if let Some(dest) = insn.destination_register() {
                        let new_type = self.compute_result_type(insn);

                        let key = (insn.address().as_u64(), dest);
                        let old_type = self.register_types.get(&key).cloned();

                        if old_type != Some(new_type.clone()) {
                            self.register_types.insert(key, new_type);
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    fn compute_result_type(&self, insn: &Instruction) -> InferredType {
        if insn.is_load() {
            if let Some(mem) = insn.memory_operand() {
                return match mem.size {
                    1 => InferredType::Integer(IntegerType::U8),
                    2 => InferredType::Integer(IntegerType::U16),
                    4 => InferredType::Integer(IntegerType::U32),
                    8 => InferredType::Integer(IntegerType::U64),
                    _ => InferredType::Unknown,
                };
            }
        }

        if insn.is_float() {
            return InferredType::Float(FloatType::F64);
        }

        if insn.is_arithmetic() || insn.is_logical() {
            return InferredType::Integer(IntegerType::U64);
        }

        if insn.is_compare() {
            return InferredType::Boolean;
        }

        InferredType::Unknown
    }

    pub fn infer_function_signature(&self, entry: Address, cfg: &ControlFlowGraph) -> FunctionSignature {
        let mut param_types = Vec::new();

        for i in 0..8u8 {
            if let Some(ty) = self.register_types.get(&(entry.as_u64(), i)) {
                param_types.push(ty.clone());
            } else {
                break;
            }
        }

        let mut return_type = Box::new(InferredType::Void);
        for block in cfg.blocks() {
            if block.has_return() {
                if let Some(ty) = self.register_types.get(&(block.end().as_u64() - 4, 0)) {
                    return_type = Box::new(ty.clone());
                    break;
                }
            }
        }

        FunctionSignature {
            return_type,
            param_types,
            is_variadic: false,
        }
    }

    pub fn clear(&mut self) {
        self.type_info.clear();
        self.register_types.clear();
    }
}

impl InferredType {
    pub fn size(&self) -> usize {
        match self {
            InferredType::Unknown => 0,
            InferredType::Integer(int_ty) => int_ty.size(),
            InferredType::Float(float_ty) => float_ty.size(),
            InferredType::Pointer(_) => 8,
            InferredType::Array(elem, count) => elem.size() * count,
            InferredType::Struct(fields) => fields.iter().map(|(_, t)| t.size()).sum(),
            InferredType::Function(_) => 8,
            InferredType::String => 8,
            InferredType::Boolean => 1,
            InferredType::Void => 0,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            InferredType::Unknown => 1,
            InferredType::Integer(int_ty) => int_ty.size(),
            InferredType::Float(float_ty) => float_ty.size(),
            InferredType::Pointer(_) => 8,
            InferredType::Array(elem, _) => elem.alignment(),
            InferredType::Struct(fields) => {
                fields.iter().map(|(_, t)| t.alignment()).max().unwrap_or(1)
            }
            InferredType::Function(_) => 8,
            InferredType::String => 8,
            InferredType::Boolean => 1,
            InferredType::Void => 1,
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, InferredType::Pointer(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, InferredType::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, InferredType::Float(_))
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self, InferredType::Array(_, _) | InferredType::Struct(_))
    }

    pub fn pointee_type(&self) -> Option<&InferredType> {
        match self {
            InferredType::Pointer(inner) => Some(inner),
            _ => None,
        }
    }
}

impl IntegerType {
    pub fn size(self) -> usize {
        match self {
            IntegerType::I8 | IntegerType::U8 => 1,
            IntegerType::I16 | IntegerType::U16 => 2,
            IntegerType::I32 | IntegerType::U32 => 4,
            IntegerType::I64 | IntegerType::U64 => 8,
        }
    }

    pub fn is_signed(self) -> bool {
        matches!(self, IntegerType::I8 | IntegerType::I16 | IntegerType::I32 | IntegerType::I64)
    }
}

impl FloatType {
    pub fn size(self) -> usize {
        match self {
            FloatType::F32 => 4,
            FloatType::F64 => 8,
        }
    }
}

impl std::fmt::Display for InferredType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferredType::Unknown => write!(f, "unknown"),
            InferredType::Integer(int_ty) => write!(f, "{:?}", int_ty),
            InferredType::Float(float_ty) => write!(f, "{:?}", float_ty),
            InferredType::Pointer(inner) => write!(f, "*{}", inner),
            InferredType::Array(elem, count) => write!(f, "[{}; {}]", elem, count),
            InferredType::Struct(fields) => {
                write!(f, "struct {{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
            InferredType::Function(sig) => {
                write!(f, "fn(")?;
                for (i, ty) in sig.param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                if sig.is_variadic {
                    write!(f, ", ...")?;
                }
                write!(f, ") -> {}", sig.return_type)
            }
            InferredType::String => write!(f, "string"),
            InferredType::Boolean => write!(f, "bool"),
            InferredType::Void => write!(f, "void"),
        }
    }
}
