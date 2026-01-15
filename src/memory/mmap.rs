// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub struct MmapMemory {
    mmap: Arc<Mmap>,
    base_address: Address,
}

impl MmapMemory {
    pub fn from_file<P: AsRef<Path>>(path: P, base_address: Address) -> Result<Self, MemoryError> {
        let file = File::open(path)
            .map_err(|e| MemoryError::Io(e))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| MemoryError::Io(e))?;
        Ok(Self {
            mmap: Arc::new(mmap),
            base_address,
        })
    }

    pub fn from_mmap(mmap: Arc<Mmap>, base_address: Address) -> Self {
        Self { mmap, base_address }
    }

    pub fn base_address(&self) -> Address {
        self.base_address
    }

    pub fn size(&self) -> usize {
        self.mmap.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.mmap.as_ref()
    }

    fn offset(&self, addr: Address) -> Result<usize, MemoryError> {
        let offset = (addr.as_u64() - self.base_address.as_u64()) as usize;
        if offset >= self.mmap.len() {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        Ok(offset)
    }
}

impl MemoryReader for MmapMemory {
    fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        let offset = self.offset(addr)?;
        if offset + len > self.mmap.len() {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        Ok(self.mmap[offset..offset + len].to_vec())
    }

    fn read_u8(&self, addr: Address) -> Result<u8, MemoryError> {
        let offset = self.offset(addr)?;
        Ok(self.mmap[offset])
    }

    fn read_u16(&self, addr: Address) -> Result<u16, MemoryError> {
        let offset = self.offset(addr)?;
        let bytes = &self.mmap[offset..offset + 2];
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&self, addr: Address) -> Result<u32, MemoryError> {
        let offset = self.offset(addr)?;
        let bytes = &self.mmap[offset..offset + 4];
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&self, addr: Address) -> Result<u64, MemoryError> {
        let offset = self.offset(addr)?;
        let bytes = &self.mmap[offset..offset + 8];
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_i8(&self, addr: Address) -> Result<i8, MemoryError> {
        Ok(self.read_u8(addr)? as i8)
    }

    fn read_i16(&self, addr: Address) -> Result<i16, MemoryError> {
        Ok(self.read_u16(addr)? as i16)
    }

    fn read_i32(&self, addr: Address) -> Result<i32, MemoryError> {
        Ok(self.read_u32(addr)? as i32)
    }

    fn read_i64(&self, addr: Address) -> Result<i64, MemoryError> {
        Ok(self.read_u64(addr)? as i64)
    }

    fn read_ptr(&self, addr: Address) -> Result<Address, MemoryError> {
        Ok(Address::new(self.read_u64(addr)?))
    }

    fn read_string(&self, addr: Address, max_len: usize) -> Result<String, MemoryError> {
        let offset = self.offset(addr)?;
        let len = (max_len.min(self.mmap.len() - offset)).min(4096);
        let bytes = &self.mmap[offset..offset + len];
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..null_pos].to_vec())
            .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    fn read_c_string(&self, addr: Address) -> Result<String, MemoryError> {
        let offset = self.offset(addr)?;
        let mut bytes = Vec::new();
        for i in offset..self.mmap.len().min(offset + 4096) {
            let byte = self.mmap[i];
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        String::from_utf8(bytes)
            .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }
}
