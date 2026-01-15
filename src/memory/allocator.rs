// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryRange};
use std::collections::BTreeMap;

pub struct MemoryAllocator {
    free_blocks: BTreeMap<u64, u64>,
    allocated_blocks: BTreeMap<u64, u64>,
}

impl MemoryAllocator {
    pub fn new() -> Self {
        Self {
            free_blocks: BTreeMap::new(),
            allocated_blocks: BTreeMap::new(),
        }
    }

    pub fn add_free_block(&mut self, start: Address, size: u64) {
        self.free_blocks.insert(start.as_u64(), size);
    }

    pub fn allocate(&mut self, size: u64, alignment: usize) -> Option<Address> {
        let aligned_size = (size + alignment as u64 - 1) & !(alignment as u64 - 1);
        for (start, block_size) in &self.free_blocks.clone() {
            let start_addr = Address::new(*start);
            let aligned_start = start_addr.align_up(alignment);
            let aligned_offset = aligned_start.as_u64() - start_addr.as_u64();
            if aligned_offset + aligned_size <= *block_size {
                let allocated_start = aligned_start.as_u64();
                let allocated_size = aligned_size;
                self.free_blocks.remove(start);
                if aligned_offset > 0 {
                    self.free_blocks.insert(*start, aligned_offset);
                }
                if aligned_offset + allocated_size < *block_size {
                    self.free_blocks.insert(allocated_start + allocated_size, *block_size - aligned_offset - allocated_size);
                }
                self.allocated_blocks.insert(allocated_start, allocated_size);
                return Some(Address::new(allocated_start));
            }
        }
        None
    }

    pub fn deallocate(&mut self, addr: Address, size: u64) -> Result<(), MemoryError> {
        let addr_u64 = addr.as_u64();
        if let Some(allocated_size) = self.allocated_blocks.remove(&addr_u64) {
            if allocated_size != size {
                return Err(MemoryError::InvalidRange);
            }
            self.free_blocks.insert(addr_u64, size);
            self.merge_free_blocks();
            Ok(())
        } else {
            Err(MemoryError::InvalidAddress(format!("Address {} not allocated", addr)))
        }
    }

    fn merge_free_blocks(&mut self) {
        let mut merged = BTreeMap::new();
        let mut current: Option<(u64, u64)> = None;
        for (start, size) in &self.free_blocks {
            if let Some((cur_start, cur_size)) = current {
                if cur_start + cur_size == *start {
                    current = Some((cur_start, cur_size + size));
                } else {
                    merged.insert(cur_start, cur_size);
                    current = Some((*start, *size));
                }
            } else {
                current = Some((*start, *size));
            }
        }
        if let Some((start, size)) = current {
            merged.insert(start, size);
        }
        self.free_blocks = merged;
    }

    pub fn is_allocated(&self, addr: Address) -> bool {
        self.allocated_blocks.contains_key(&addr.as_u64())
    }

    pub fn get_allocated_size(&self, addr: Address) -> Option<u64> {
        self.allocated_blocks.get(&addr.as_u64()).copied()
    }
}

impl Default for MemoryAllocator {
    fn default() -> Self {
        Self::new()
    }
}
