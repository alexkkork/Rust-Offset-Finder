// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryRegion};

pub trait MemoryReader: Send + Sync {
    fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError>;
    fn read_u8(&self, addr: Address) -> Result<u8, MemoryError>;
    fn read_u16(&self, addr: Address) -> Result<u16, MemoryError>;
    fn read_u32(&self, addr: Address) -> Result<u32, MemoryError>;
    fn read_u64(&self, addr: Address) -> Result<u64, MemoryError>;
    fn read_i8(&self, addr: Address) -> Result<i8, MemoryError>;
    fn read_i16(&self, addr: Address) -> Result<i16, MemoryError>;
    fn read_i32(&self, addr: Address) -> Result<i32, MemoryError>;
    fn read_i64(&self, addr: Address) -> Result<i64, MemoryError>;
    fn read_ptr(&self, addr: Address) -> Result<Address, MemoryError>;
    fn read_string(&self, addr: Address, max_len: usize) -> Result<String, MemoryError>;
    fn read_c_string(&self, addr: Address) -> Result<String, MemoryError>;
    fn get_base_address(&self) -> Address;
    fn get_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError>;
}

pub trait MemoryWriter: Send + Sync {
    fn write_bytes(&mut self, addr: Address, data: &[u8]) -> Result<(), MemoryError>;
    fn write_u8(&mut self, addr: Address, value: u8) -> Result<(), MemoryError>;
    fn write_u16(&mut self, addr: Address, value: u16) -> Result<(), MemoryError>;
    fn write_u32(&mut self, addr: Address, value: u32) -> Result<(), MemoryError>;
    fn write_u64(&mut self, addr: Address, value: u64) -> Result<(), MemoryError>;
    fn write_ptr(&mut self, addr: Address, value: Address) -> Result<(), MemoryError>;
}
