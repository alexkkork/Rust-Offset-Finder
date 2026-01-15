// Tue Jan 13 2026 - Alex

use crate::memory::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuauType {
    Nil,
    Boolean,
    LightUserData,
    Number,
    Vector,
    String,
    Table,
    Function,
    UserData,
    Thread,
    Unknown(u8),
}

impl LuauType {
    pub fn from_tag(tag: TypeTag) -> Self {
        match tag {
            TypeTag::Nil => LuauType::Nil,
            TypeTag::Boolean => LuauType::Boolean,
            TypeTag::LightUserData => LuauType::LightUserData,
            TypeTag::Number => LuauType::Number,
            TypeTag::Vector => LuauType::Vector,
            TypeTag::String => LuauType::String,
            TypeTag::Table => LuauType::Table,
            TypeTag::Function => LuauType::Function,
            TypeTag::UserData => LuauType::UserData,
            TypeTag::Thread => LuauType::Thread,
            TypeTag::Unknown(v) => LuauType::Unknown(v),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LuauType::Nil => "nil",
            LuauType::Boolean => "boolean",
            LuauType::LightUserData => "lightuserdata",
            LuauType::Number => "number",
            LuauType::Vector => "vector",
            LuauType::String => "string",
            LuauType::Table => "table",
            LuauType::Function => "function",
            LuauType::UserData => "userdata",
            LuauType::Thread => "thread",
            LuauType::Unknown(_) => "unknown",
        }
    }

    pub fn is_gc_collectable(&self) -> bool {
        matches!(self,
            LuauType::String |
            LuauType::Table |
            LuauType::Function |
            LuauType::UserData |
            LuauType::Thread
        )
    }

    pub fn is_value_type(&self) -> bool {
        matches!(self,
            LuauType::Nil |
            LuauType::Boolean |
            LuauType::Number |
            LuauType::Vector |
            LuauType::LightUserData
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTag {
    Nil,
    Boolean,
    LightUserData,
    Number,
    Vector,
    String,
    Table,
    Function,
    UserData,
    Thread,
    Unknown(u8),
}

impl TypeTag {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => TypeTag::Nil,
            1 => TypeTag::Boolean,
            2 => TypeTag::LightUserData,
            3 => TypeTag::Number,
            4 => TypeTag::Vector,
            5 => TypeTag::String,
            6 => TypeTag::Table,
            7 => TypeTag::Function,
            8 => TypeTag::UserData,
            9 => TypeTag::Thread,
            _ => TypeTag::Unknown(value),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TypeTag::Nil => 0,
            TypeTag::Boolean => 1,
            TypeTag::LightUserData => 2,
            TypeTag::Number => 3,
            TypeTag::Vector => 4,
            TypeTag::String => 5,
            TypeTag::Table => 6,
            TypeTag::Function => 7,
            TypeTag::UserData => 8,
            TypeTag::Thread => 9,
            TypeTag::Unknown(v) => *v,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TValue {
    pub value: TValueData,
    pub tt: TypeTag,
}

impl TValue {
    pub fn nil() -> Self {
        Self {
            value: TValueData::Nil,
            tt: TypeTag::Nil,
        }
    }

    pub fn boolean(b: bool) -> Self {
        Self {
            value: TValueData::Boolean(b),
            tt: TypeTag::Boolean,
        }
    }

    pub fn number(n: f64) -> Self {
        Self {
            value: TValueData::Number(n),
            tt: TypeTag::Number,
        }
    }

    pub fn vector(x: f32, y: f32, z: f32) -> Self {
        Self {
            value: TValueData::Vector(x, y, z),
            tt: TypeTag::Vector,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.tt == TypeTag::Nil
    }

    pub fn is_boolean(&self) -> bool {
        self.tt == TypeTag::Boolean
    }

    pub fn is_number(&self) -> bool {
        self.tt == TypeTag::Number
    }

    pub fn is_string(&self) -> bool {
        self.tt == TypeTag::String
    }

    pub fn is_table(&self) -> bool {
        self.tt == TypeTag::Table
    }

    pub fn is_function(&self) -> bool {
        self.tt == TypeTag::Function
    }

    pub fn is_userdata(&self) -> bool {
        self.tt == TypeTag::UserData
    }

    pub fn is_thread(&self) -> bool {
        self.tt == TypeTag::Thread
    }

    pub fn is_vector(&self) -> bool {
        self.tt == TypeTag::Vector
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match &self.value {
            TValueData::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match &self.value {
            TValueData::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_pointer(&self) -> Option<Address> {
        match &self.value {
            TValueData::GcObject(addr) => Some(*addr),
            TValueData::LightUserData(addr) => Some(*addr),
            _ => None,
        }
    }

    pub fn as_vector(&self) -> Option<(f32, f32, f32)> {
        match &self.value {
            TValueData::Vector(x, y, z) => Some((*x, *y, *z)),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        LuauType::from_tag(self.tt).name()
    }
}

#[derive(Debug, Clone)]
pub enum TValueData {
    Nil,
    Boolean(bool),
    Number(f64),
    Vector(f32, f32, f32),
    LightUserData(Address),
    GcObject(Address),
}

#[derive(Debug, Clone)]
pub struct GCHeader {
    pub next: Address,
    pub tt: TypeTag,
    pub marked: u8,
    pub memcat: u8,
}

impl GCHeader {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        let next = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);

        let tt = TypeTag::from_u8(data[8]);
        let marked = data[9];
        let memcat = data[10];

        Some(Self {
            next: Address::new(next),
            tt,
            marked,
            memcat,
        })
    }

    pub fn is_white(&self) -> bool {
        self.marked & 0x03 != 0
    }

    pub fn is_black(&self) -> bool {
        self.marked & 0x04 != 0
    }

    pub fn is_gray(&self) -> bool {
        !self.is_white() && !self.is_black()
    }
}

#[derive(Debug, Clone)]
pub struct TableValue {
    pub gc: GCHeader,
    pub flags: u8,
    pub node_log2_size: u8,
    pub readonly: bool,
    pub safe_env: bool,
    pub array_size: u32,
    pub metatable: Option<Address>,
    pub array: Address,
    pub node: Address,
    pub last_free: Address,
}

impl TableValue {
    pub fn node_size(&self) -> usize {
        1 << self.node_log2_size
    }

    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    pub fn has_metatable(&self) -> bool {
        self.metatable.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct StringValue {
    pub gc: GCHeader,
    pub atom: u16,
    pub hash: u32,
    pub len: u32,
}

impl StringValue {
    pub fn is_atom(&self) -> bool {
        self.atom != 0xFFFF
    }
}

#[derive(Debug, Clone)]
pub struct ClosureValue {
    pub gc: GCHeader,
    pub is_c: bool,
    pub nupvalues: u8,
    pub stack_size: u8,
    pub preload: u8,
    pub env: Address,
}

impl ClosureValue {
    pub fn is_c_closure(&self) -> bool {
        self.is_c
    }

    pub fn is_lua_closure(&self) -> bool {
        !self.is_c
    }
}

#[derive(Debug, Clone)]
pub struct ProtoValue {
    pub gc: GCHeader,
    pub nups: u8,
    pub numparams: u8,
    pub is_vararg: bool,
    pub maxstacksize: u8,
    pub sizecode: u32,
    pub sizek: u32,
    pub sizep: u32,
    pub sizelineinfo: u32,
    pub code: Address,
    pub k: Address,
    pub p: Address,
    pub lineinfo: Address,
    pub source: Address,
}

impl ProtoValue {
    pub fn has_varargs(&self) -> bool {
        self.is_vararg
    }

    pub fn has_debug_info(&self) -> bool {
        self.sizelineinfo > 0
    }
}

#[derive(Debug, Clone)]
pub struct UserdataValue {
    pub gc: GCHeader,
    pub tag: u8,
    pub len: u32,
    pub metatable: Option<Address>,
}

impl UserdataValue {
    pub fn has_metatable(&self) -> bool {
        self.metatable.is_some()
    }

    pub fn data_size(&self) -> usize {
        self.len as usize
    }
}

pub struct TypeSize {
    pub tvalue: usize,
    pub gc_header: usize,
    pub string_header: usize,
    pub table_header: usize,
    pub closure_header: usize,
    pub proto_header: usize,
    pub userdata_header: usize,
    pub thread_header: usize,
}

impl Default for TypeSize {
    fn default() -> Self {
        Self {
            tvalue: 16,
            gc_header: 16,
            string_header: 24,
            table_header: 56,
            closure_header: 32,
            proto_header: 96,
            userdata_header: 24,
            thread_header: 128,
        }
    }
}
