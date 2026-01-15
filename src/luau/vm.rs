// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::opcode::LuauOpcode;
use std::sync::Arc;
use std::collections::HashMap;

pub struct VmAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl VmAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_vm_execute(&self) -> Result<Option<Address>, MemoryError> {
        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.protection.is_executable() {
                continue;
            }
        }

        Ok(None)
    }

    pub fn find_opcode_handlers(&self) -> Result<HashMap<LuauOpcode, Address>, MemoryError> {
        let mut handlers = HashMap::new();

        Ok(handlers)
    }

    pub fn find_fastcall_table(&self) -> Result<Option<Address>, MemoryError> {
        Ok(None)
    }

    pub fn analyze_dispatch_table(&self, table_addr: Address) -> Result<DispatchTableInfo, MemoryError> {
        let mut info = DispatchTableInfo::new();

        let table_size = 256;
        let entry_size = 8;

        for i in 0..table_size {
            let entry_addr = table_addr + (i * entry_size) as u64;
            let handler = self.reader.read_u64(entry_addr)?;

            if handler != 0 {
                let opcode = LuauOpcode::from_u8(i as u8);
                info.handlers.insert(opcode, Address::new(handler));
            }
        }

        Ok(info)
    }

    pub fn analyze_fastcall_handlers(&self, table_addr: Address) -> Result<FastcallTableInfo, MemoryError> {
        let mut info = FastcallTableInfo::new();

        let builtin_count = 64;
        let entry_size = 8;

        for i in 0..builtin_count {
            let entry_addr = table_addr + (i * entry_size) as u64;
            let handler = self.reader.read_u64(entry_addr)?;

            if handler != 0 {
                let builtin = BuiltinFunction::from_index(i);
                info.builtins.insert(builtin, Address::new(handler));
            }
        }

        Ok(info)
    }

    pub fn find_interrupt_handler(&self) -> Result<Option<Address>, MemoryError> {
        Ok(None)
    }

    pub fn find_debugbreak_handler(&self) -> Result<Option<Address>, MemoryError> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct DispatchTableInfo {
    pub address: Option<Address>,
    pub handlers: HashMap<LuauOpcode, Address>,
    pub size: usize,
}

impl DispatchTableInfo {
    pub fn new() -> Self {
        Self {
            address: None,
            handlers: HashMap::new(),
            size: 0,
        }
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn get_handler(&self, opcode: LuauOpcode) -> Option<Address> {
        self.handlers.get(&opcode).copied()
    }

    pub fn has_handler(&self, opcode: LuauOpcode) -> bool {
        self.handlers.contains_key(&opcode)
    }
}

impl Default for DispatchTableInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FastcallTableInfo {
    pub address: Option<Address>,
    pub builtins: HashMap<BuiltinFunction, Address>,
    pub size: usize,
}

impl FastcallTableInfo {
    pub fn new() -> Self {
        Self {
            address: None,
            builtins: HashMap::new(),
            size: 0,
        }
    }

    pub fn builtin_count(&self) -> usize {
        self.builtins.len()
    }

    pub fn get_builtin(&self, builtin: BuiltinFunction) -> Option<Address> {
        self.builtins.get(&builtin).copied()
    }
}

impl Default for FastcallTableInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinFunction {
    None,
    Assert,
    Abs,
    Acos,
    Asin,
    Atan2,
    Atan,
    Ceil,
    Cosh,
    Cos,
    Deg,
    Exp,
    Floor,
    Fmod,
    Frexp,
    Ldexp,
    Log10,
    Log,
    Max,
    Min,
    Modf,
    Pow,
    Rad,
    Sinh,
    Sin,
    Sqrt,
    Tanh,
    Tan,
    Arshift,
    Band,
    Bnot,
    Bor,
    Bxor,
    Btest,
    Extract,
    Lrotate,
    Lshift,
    Replace,
    Rrotate,
    Rshift,
    Type,
    Typeof,
    Clamp,
    Sign,
    Round,
    Rawset,
    Rawget,
    Rawequal,
    Tinsert,
    Tunpack,
    Setmetatable,
    Getmetatable,
    Unknown(usize),
}

impl BuiltinFunction {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => BuiltinFunction::None,
            1 => BuiltinFunction::Assert,
            2 => BuiltinFunction::Abs,
            3 => BuiltinFunction::Acos,
            4 => BuiltinFunction::Asin,
            5 => BuiltinFunction::Atan2,
            6 => BuiltinFunction::Atan,
            7 => BuiltinFunction::Ceil,
            8 => BuiltinFunction::Cosh,
            9 => BuiltinFunction::Cos,
            10 => BuiltinFunction::Deg,
            11 => BuiltinFunction::Exp,
            12 => BuiltinFunction::Floor,
            13 => BuiltinFunction::Fmod,
            14 => BuiltinFunction::Frexp,
            15 => BuiltinFunction::Ldexp,
            16 => BuiltinFunction::Log10,
            17 => BuiltinFunction::Log,
            18 => BuiltinFunction::Max,
            19 => BuiltinFunction::Min,
            20 => BuiltinFunction::Modf,
            21 => BuiltinFunction::Pow,
            22 => BuiltinFunction::Rad,
            23 => BuiltinFunction::Sinh,
            24 => BuiltinFunction::Sin,
            25 => BuiltinFunction::Sqrt,
            26 => BuiltinFunction::Tanh,
            27 => BuiltinFunction::Tan,
            28 => BuiltinFunction::Arshift,
            29 => BuiltinFunction::Band,
            30 => BuiltinFunction::Bnot,
            31 => BuiltinFunction::Bor,
            32 => BuiltinFunction::Bxor,
            33 => BuiltinFunction::Btest,
            34 => BuiltinFunction::Extract,
            35 => BuiltinFunction::Lrotate,
            36 => BuiltinFunction::Lshift,
            37 => BuiltinFunction::Replace,
            38 => BuiltinFunction::Rrotate,
            39 => BuiltinFunction::Rshift,
            40 => BuiltinFunction::Type,
            41 => BuiltinFunction::Typeof,
            42 => BuiltinFunction::Clamp,
            43 => BuiltinFunction::Sign,
            44 => BuiltinFunction::Round,
            45 => BuiltinFunction::Rawset,
            46 => BuiltinFunction::Rawget,
            47 => BuiltinFunction::Rawequal,
            48 => BuiltinFunction::Tinsert,
            49 => BuiltinFunction::Tunpack,
            50 => BuiltinFunction::Setmetatable,
            51 => BuiltinFunction::Getmetatable,
            _ => BuiltinFunction::Unknown(index),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuiltinFunction::None => "none",
            BuiltinFunction::Assert => "assert",
            BuiltinFunction::Abs => "math.abs",
            BuiltinFunction::Acos => "math.acos",
            BuiltinFunction::Asin => "math.asin",
            BuiltinFunction::Atan2 => "math.atan2",
            BuiltinFunction::Atan => "math.atan",
            BuiltinFunction::Ceil => "math.ceil",
            BuiltinFunction::Cosh => "math.cosh",
            BuiltinFunction::Cos => "math.cos",
            BuiltinFunction::Deg => "math.deg",
            BuiltinFunction::Exp => "math.exp",
            BuiltinFunction::Floor => "math.floor",
            BuiltinFunction::Fmod => "math.fmod",
            BuiltinFunction::Frexp => "math.frexp",
            BuiltinFunction::Ldexp => "math.ldexp",
            BuiltinFunction::Log10 => "math.log10",
            BuiltinFunction::Log => "math.log",
            BuiltinFunction::Max => "math.max",
            BuiltinFunction::Min => "math.min",
            BuiltinFunction::Modf => "math.modf",
            BuiltinFunction::Pow => "math.pow",
            BuiltinFunction::Rad => "math.rad",
            BuiltinFunction::Sinh => "math.sinh",
            BuiltinFunction::Sin => "math.sin",
            BuiltinFunction::Sqrt => "math.sqrt",
            BuiltinFunction::Tanh => "math.tanh",
            BuiltinFunction::Tan => "math.tan",
            BuiltinFunction::Arshift => "bit32.arshift",
            BuiltinFunction::Band => "bit32.band",
            BuiltinFunction::Bnot => "bit32.bnot",
            BuiltinFunction::Bor => "bit32.bor",
            BuiltinFunction::Bxor => "bit32.bxor",
            BuiltinFunction::Btest => "bit32.btest",
            BuiltinFunction::Extract => "bit32.extract",
            BuiltinFunction::Lrotate => "bit32.lrotate",
            BuiltinFunction::Lshift => "bit32.lshift",
            BuiltinFunction::Replace => "bit32.replace",
            BuiltinFunction::Rrotate => "bit32.rrotate",
            BuiltinFunction::Rshift => "bit32.rshift",
            BuiltinFunction::Type => "type",
            BuiltinFunction::Typeof => "typeof",
            BuiltinFunction::Clamp => "math.clamp",
            BuiltinFunction::Sign => "math.sign",
            BuiltinFunction::Round => "math.round",
            BuiltinFunction::Rawset => "rawset",
            BuiltinFunction::Rawget => "rawget",
            BuiltinFunction::Rawequal => "rawequal",
            BuiltinFunction::Tinsert => "table.insert",
            BuiltinFunction::Tunpack => "table.unpack",
            BuiltinFunction::Setmetatable => "setmetatable",
            BuiltinFunction::Getmetatable => "getmetatable",
            BuiltinFunction::Unknown(_) => "unknown",
        }
    }
}

pub struct VmState {
    pub pc: usize,
    pub stack_size: usize,
    pub call_depth: usize,
    pub upvalue_count: usize,
    pub is_yielded: bool,
    pub is_running: bool,
    pub error_state: Option<String>,
}

impl VmState {
    pub fn new() -> Self {
        Self {
            pc: 0,
            stack_size: 0,
            call_depth: 0,
            upvalue_count: 0,
            is_yielded: false,
            is_running: false,
            error_state: None,
        }
    }
}

impl Default for VmState {
    fn default() -> Self {
        Self::new()
    }
}
