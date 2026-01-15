// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MemoryCache {
    cache: RwLock<HashMap<u64, Vec<u8>>>,
    reader: Arc<dyn MemoryReader>,
    max_size: usize,
    block_size: usize,
}

impl MemoryCache {
    pub fn new(reader: Arc<dyn MemoryReader>, max_size: usize, block_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            reader,
            max_size,
            block_size,
        }
    }

    pub fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        let start_addr = addr.align_down(self.block_size);
        let start_offset = addr.as_u64() - start_addr.as_u64();
        let end_addr = (addr + len as u64).align_up(self.block_size);
        let blocks_needed = ((end_addr.as_u64() - start_addr.as_u64()) / self.block_size as u64) as usize;

        let mut result = Vec::with_capacity(len);

        for i in 0..blocks_needed {
            let block_addr = start_addr + (i * self.block_size) as u64;
            let block_key = block_addr.as_u64();

            let block_data = {
                let cache_read = self.cache.read();
                if let Some(cached) = cache_read.get(&block_key) {
                    cached.clone()
                } else {
                    drop(cache_read);
                    let data = self.reader.read_bytes(block_addr, self.block_size)?;
                    let mut cache_write = self.cache.write();
                    if cache_write.len() >= self.max_size {
                        cache_write.clear();
                    }
                    cache_write.insert(block_key, data.clone());
                    data
                }
            };

            let block_start = if i == 0 { start_offset as usize } else { 0 };
            let block_end = if i == blocks_needed - 1 {
                ((end_addr.as_u64() - block_addr.as_u64()) as usize).min(self.block_size)
            } else {
                self.block_size
            };

            let needed_len = (len - result.len()).min(block_end - block_start);
            if block_start + needed_len <= block_data.len() {
                result.extend_from_slice(&block_data[block_start..block_start + needed_len]);
            }
        }

        Ok(result)
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }

    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    pub fn invalidate(&self, addr: Address) {
        let block_addr = addr.align_down(self.block_size);
        self.cache.write().remove(&block_addr.as_u64());
    }

    pub fn invalidate_range(&self, start: Address, end: Address) {
        let start_block = start.align_down(self.block_size);
        let end_block = end.align_up(self.block_size);
        let mut cache = self.cache.write();
        let mut addr = start_block.as_u64();
        while addr < end_block.as_u64() {
            cache.remove(&addr);
            addr += self.block_size as u64;
        }
    }

    pub fn prefetch(&self, addr: Address, len: usize) -> Result<(), MemoryError> {
        let _ = self.read_bytes(addr, len)?;
        Ok(())
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

impl MemoryReader for MemoryCache {
    fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        MemoryCache::read_bytes(self, addr, len)
    }

    fn read_u8(&self, addr: Address) -> Result<u8, MemoryError> {
        let bytes = self.read_bytes(addr, 1)?;
        Ok(bytes[0])
    }

    fn read_u16(&self, addr: Address) -> Result<u16, MemoryError> {
        let bytes = self.read_bytes(addr, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&self, addr: Address) -> Result<u32, MemoryError> {
        let bytes = self.read_bytes(addr, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&self, addr: Address) -> Result<u64, MemoryError> {
        let bytes = self.read_bytes(addr, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
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
        let bytes = self.read_bytes(addr, max_len)?;
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..null_pos].to_vec())
            .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    fn read_c_string(&self, addr: Address) -> Result<String, MemoryError> {
        let mut bytes = Vec::new();
        let mut current = addr;
        loop {
            let byte = self.read_u8(current)?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            current = current + 1;
            if bytes.len() > 4096 {
                return Err(MemoryError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "String too long",
                )));
            }
        }
        String::from_utf8(bytes)
            .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }
}
