// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::luau::types::{TValue, TValueData, TypeTag};
use std::sync::Arc;
use std::collections::HashMap;

pub struct LuauApi {
    reader: Arc<dyn MemoryReader>,
}

impl LuauApi {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn read_top(&self, state: Address) -> Result<Address, MemoryError> {
        let top = self.reader.read_u64(state + 0x10)?;
        Ok(Address::new(top))
    }

    pub fn read_base(&self, state: Address) -> Result<Address, MemoryError> {
        let base = self.reader.read_u64(state + 0x08)?;
        Ok(Address::new(base))
    }

    pub fn read_stack(&self, state: Address) -> Result<Address, MemoryError> {
        let stack = self.reader.read_u64(state + 0x18)?;
        Ok(Address::new(stack))
    }

    pub fn get_stack_size(&self, state: Address) -> Result<i64, MemoryError> {
        let top = self.read_top(state)?.as_u64();
        let base = self.read_base(state)?.as_u64();

        if top >= base {
            Ok(((top - base) / 16) as i64)
        } else {
            Ok(0)
        }
    }

    pub fn read_stack_value(&self, state: Address, index: i32) -> Result<TValue, MemoryError> {
        let value_addr = self.index_to_address(state, index)?;
        self.read_tvalue(value_addr)
    }

    pub fn index_to_address(&self, state: Address, index: i32) -> Result<Address, MemoryError> {
        if index > 0 {
            let base = self.read_base(state)?;
            Ok(base + ((index - 1) as u64 * 16))
        } else if index < 0 {
            let top = self.read_top(state)?;
            Ok(top + (index as u64 * 16))
        } else {
            Err(MemoryError::Other("Invalid stack index 0".to_string()))
        }
    }

    pub fn read_tvalue(&self, addr: Address) -> Result<TValue, MemoryError> {
        let data = self.reader.read_bytes(addr, 16)?;

        let value_bytes: [u8; 8] = [
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ];

        let tt = TypeTag::from_u8(data[8]);

        let value = match tt {
            TypeTag::Nil => TValueData::Nil,
            TypeTag::Boolean => TValueData::Boolean(value_bytes[0] != 0),
            TypeTag::Number => TValueData::Number(f64::from_le_bytes(value_bytes)),
            TypeTag::Vector => {
                let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let z_bytes = self.reader.read_bytes(addr + 12, 4)?;
                let z = f32::from_le_bytes([z_bytes[0], z_bytes[1], z_bytes[2], z_bytes[3]]);
                TValueData::Vector(x, y, z)
            }
            TypeTag::LightUserData => {
                let ptr = u64::from_le_bytes(value_bytes);
                TValueData::LightUserData(Address::new(ptr))
            }
            _ => {
                let ptr = u64::from_le_bytes(value_bytes);
                TValueData::GcObject(Address::new(ptr))
            }
        };

        Ok(TValue { value, tt })
    }

    pub fn read_string(&self, str_addr: Address) -> Result<String, MemoryError> {
        let len = self.reader.read_u32(str_addr + 0x10)? as usize;
        if len > 0x100000 {
            return Err(MemoryError::Other("String too long".to_string()));
        }

        let data = self.reader.read_bytes(str_addr + 0x18, len)?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    pub fn read_table_array(&self, table_addr: Address) -> Result<Vec<TValue>, MemoryError> {
        let mut values = Vec::new();

        let array_ptr = self.reader.read_u64(table_addr + 0x18)?;
        let array_size = self.reader.read_u32(table_addr + 0x28)? as usize;

        if array_ptr == 0 || array_size == 0 {
            return Ok(values);
        }

        let array_addr = Address::new(array_ptr);

        for i in 0..array_size.min(1000) {
            let val = self.read_tvalue(array_addr + (i as u64 * 16))?;
            values.push(val);
        }

        Ok(values)
    }

    pub fn read_table_pairs(&self, table_addr: Address) -> Result<Vec<(TValue, TValue)>, MemoryError> {
        let mut pairs = Vec::new();

        let array = self.read_table_array(table_addr)?;
        for (i, val) in array.into_iter().enumerate() {
            if !val.is_nil() {
                let key = TValue::number((i + 1) as f64);
                pairs.push((key, val));
            }
        }

        let node_ptr = self.reader.read_u64(table_addr + 0x20)?;
        let log2_size = self.reader.read_u8(table_addr + 0x09)?;

        if node_ptr != 0 && log2_size > 0 {
            let node_count = 1usize << log2_size;
            let node_addr = Address::new(node_ptr);
            let node_size = 32;

            for i in 0..node_count.min(1000) {
                let key = self.read_tvalue(node_addr + (i as u64 * node_size))?;
                let val = self.read_tvalue(node_addr + (i as u64 * node_size) + 16)?;

                if !key.is_nil() && !val.is_nil() {
                    pairs.push((key, val));
                }
            }
        }

        Ok(pairs)
    }

    pub fn read_closure_info(&self, closure_addr: Address) -> Result<ClosureInfo, MemoryError> {
        let is_c = self.reader.read_u8(closure_addr + 0x08)? != 0;
        let nupvalues = self.reader.read_u8(closure_addr + 0x09)?;

        let env_ptr = self.reader.read_u64(closure_addr + 0x18)?;

        if is_c {
            let func_ptr = self.reader.read_u64(closure_addr + 0x20)?;
            Ok(ClosureInfo {
                is_c: true,
                nupvalues,
                env: if env_ptr != 0 { Some(Address::new(env_ptr)) } else { None },
                c_function: Some(Address::new(func_ptr)),
                proto: None,
            })
        } else {
            let proto_ptr = self.reader.read_u64(closure_addr + 0x20)?;
            Ok(ClosureInfo {
                is_c: false,
                nupvalues,
                env: if env_ptr != 0 { Some(Address::new(env_ptr)) } else { None },
                c_function: None,
                proto: Some(Address::new(proto_ptr)),
            })
        }
    }

    pub fn read_proto_info(&self, proto_addr: Address) -> Result<ProtoInfo, MemoryError> {
        let nups = self.reader.read_u8(proto_addr + 0x08)?;
        let numparams = self.reader.read_u8(proto_addr + 0x09)?;
        let is_vararg = self.reader.read_u8(proto_addr + 0x0A)? != 0;
        let maxstacksize = self.reader.read_u8(proto_addr + 0x0B)?;

        let sizecode = self.reader.read_u32(proto_addr + 0x10)?;
        let sizek = self.reader.read_u32(proto_addr + 0x14)?;
        let sizep = self.reader.read_u32(proto_addr + 0x18)?;

        let code_ptr = self.reader.read_u64(proto_addr + 0x20)?;
        let k_ptr = self.reader.read_u64(proto_addr + 0x28)?;

        Ok(ProtoInfo {
            nups,
            numparams,
            is_vararg,
            maxstacksize,
            sizecode,
            sizek,
            sizep,
            code: Address::new(code_ptr),
            constants: Address::new(k_ptr),
        })
    }

    pub fn read_global_table(&self, state: Address) -> Result<Address, MemoryError> {
        let gt_ptr = self.reader.read_u64(state + 0x38)?;
        Ok(Address::new(gt_ptr))
    }

    pub fn read_registry(&self, state: Address) -> Result<Address, MemoryError> {
        let global_state = self.reader.read_u64(state + 0x40)?;
        let registry = self.reader.read_u64(Address::new(global_state) + 0x28)?;
        Ok(Address::new(registry))
    }

    pub fn enumerate_globals(&self, state: Address) -> Result<HashMap<String, TValue>, MemoryError> {
        let mut globals = HashMap::new();

        let gt = self.read_global_table(state)?;
        let pairs = self.read_table_pairs(gt)?;

        for (key, val) in pairs {
            if key.is_string() {
                if let TValueData::GcObject(str_addr) = key.value {
                    let name = self.read_string(str_addr)?;
                    globals.insert(name, val);
                }
            }
        }

        Ok(globals)
    }

    pub fn get_metatable(&self, table_addr: Address) -> Result<Option<Address>, MemoryError> {
        let mt_ptr = self.reader.read_u64(table_addr + 0x28)?;
        if mt_ptr != 0 {
            Ok(Some(Address::new(mt_ptr)))
        } else {
            Ok(None)
        }
    }

    pub fn read_userdata_value(&self, ud_addr: Address) -> Result<UserdataInfo, MemoryError> {
        let tag = self.reader.read_u8(ud_addr + 0x08)?;
        let len = self.reader.read_u32(ud_addr + 0x0C)?;
        let mt_ptr = self.reader.read_u64(ud_addr + 0x10)?;

        Ok(UserdataInfo {
            tag,
            len,
            metatable: if mt_ptr != 0 { Some(Address::new(mt_ptr)) } else { None },
            data: ud_addr + 0x18,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClosureInfo {
    pub is_c: bool,
    pub nupvalues: u8,
    pub env: Option<Address>,
    pub c_function: Option<Address>,
    pub proto: Option<Address>,
}

impl ClosureInfo {
    pub fn is_c_closure(&self) -> bool {
        self.is_c
    }

    pub fn is_lua_closure(&self) -> bool {
        !self.is_c
    }
}

#[derive(Debug, Clone)]
pub struct ProtoInfo {
    pub nups: u8,
    pub numparams: u8,
    pub is_vararg: bool,
    pub maxstacksize: u8,
    pub sizecode: u32,
    pub sizek: u32,
    pub sizep: u32,
    pub code: Address,
    pub constants: Address,
}

impl ProtoInfo {
    pub fn has_upvalues(&self) -> bool {
        self.nups > 0
    }

    pub fn has_nested_protos(&self) -> bool {
        self.sizep > 0
    }
}

#[derive(Debug, Clone)]
pub struct UserdataInfo {
    pub tag: u8,
    pub len: u32,
    pub metatable: Option<Address>,
    pub data: Address,
}

impl UserdataInfo {
    pub fn has_metatable(&self) -> bool {
        self.metatable.is_some()
    }

    pub fn data_size(&self) -> usize {
        self.len as usize
    }
}
