// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

pub struct ClassDescriptor {
    pub address: Address,
    pub vtable: Address,
    pub name_ptr: Address,
    pub name: String,
    pub parent_descriptor: Option<Address>,
    pub properties_ptr: Option<Address>,
    pub events_ptr: Option<Address>,
    pub callbacks_ptr: Option<Address>,
}

pub struct DescriptorReader {
    reader: Arc<dyn MemoryReader>,
}

impl DescriptorReader {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn read_descriptor(&self, addr: Address) -> Option<ClassDescriptor> {
        let bytes = self.reader.read_bytes(addr, 128).ok()?;

        let vtable = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let name_ptr = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);

        if vtable < 0x100000000 || vtable > 0x7FFFFFFFFFFF {
            return None;
        }

        if name_ptr < 0x100000000 || name_ptr > 0x7FFFFFFFFFFF {
            return None;
        }

        let name = self.read_string(Address::new(name_ptr))?;

        let parent_ptr = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);

        let parent_descriptor = if parent_ptr >= 0x100000000 && parent_ptr <= 0x7FFFFFFFFFFF {
            Some(Address::new(parent_ptr))
        } else {
            None
        };

        let properties_ptr_raw = u64::from_le_bytes([
            bytes[32], bytes[33], bytes[34], bytes[35],
            bytes[36], bytes[37], bytes[38], bytes[39],
        ]);

        let properties_ptr = if properties_ptr_raw >= 0x100000000 && properties_ptr_raw <= 0x7FFFFFFFFFFF {
            Some(Address::new(properties_ptr_raw))
        } else {
            None
        };

        let events_ptr_raw = u64::from_le_bytes([
            bytes[40], bytes[41], bytes[42], bytes[43],
            bytes[44], bytes[45], bytes[46], bytes[47],
        ]);

        let events_ptr = if events_ptr_raw >= 0x100000000 && events_ptr_raw <= 0x7FFFFFFFFFFF {
            Some(Address::new(events_ptr_raw))
        } else {
            None
        };

        let callbacks_ptr_raw = u64::from_le_bytes([
            bytes[48], bytes[49], bytes[50], bytes[51],
            bytes[52], bytes[53], bytes[54], bytes[55],
        ]);

        let callbacks_ptr = if callbacks_ptr_raw >= 0x100000000 && callbacks_ptr_raw <= 0x7FFFFFFFFFFF {
            Some(Address::new(callbacks_ptr_raw))
        } else {
            None
        };

        Some(ClassDescriptor {
            address: addr,
            vtable: Address::new(vtable),
            name_ptr: Address::new(name_ptr),
            name,
            parent_descriptor,
            properties_ptr,
            events_ptr,
            callbacks_ptr,
        })
    }

    fn read_string(&self, addr: Address) -> Option<String> {
        let bytes = self.reader.read_bytes(addr, 256).ok()?;

        let null_pos = bytes.iter().position(|&b| b == 0)?;

        String::from_utf8(bytes[..null_pos].to_vec()).ok()
    }

    pub fn read_property_list(&self, list_addr: Address) -> Vec<PropertyDescriptor> {
        let mut properties = Vec::new();

        let mut current = list_addr;

        for _ in 0..1000 {
            if let Ok(bytes) = self.reader.read_bytes(current, 8) {
                let entry_ptr = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                if entry_ptr < 0x100000000 || entry_ptr > 0x7FFFFFFFFFFF {
                    break;
                }

                if let Some(prop) = self.read_property_descriptor(Address::new(entry_ptr)) {
                    properties.push(prop);
                }

                current = current + 8;
            } else {
                break;
            }
        }

        properties
    }

    fn read_property_descriptor(&self, addr: Address) -> Option<PropertyDescriptor> {
        let bytes = self.reader.read_bytes(addr, 64).ok()?;

        let name_ptr = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        if name_ptr < 0x100000000 || name_ptr > 0x7FFFFFFFFFFF {
            return None;
        }

        let name = self.read_string(Address::new(name_ptr))?;

        let getter_ptr = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);

        let setter_ptr = u64::from_le_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27],
            bytes[28], bytes[29], bytes[30], bytes[31],
        ]);

        let getter = if getter_ptr >= 0x100000000 && getter_ptr <= 0x7FFFFFFFFFFF {
            Some(Address::new(getter_ptr))
        } else {
            None
        };

        let setter = if setter_ptr >= 0x100000000 && setter_ptr <= 0x7FFFFFFFFFFF {
            Some(Address::new(setter_ptr))
        } else {
            None
        };

        Some(PropertyDescriptor {
            address: addr,
            name,
            getter,
            setter,
            property_type: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub address: Address,
    pub name: String,
    pub getter: Option<Address>,
    pub setter: Option<Address>,
    pub property_type: Option<String>,
}
