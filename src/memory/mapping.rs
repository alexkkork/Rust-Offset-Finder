// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader, MemoryRegion};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MemoryMapping {
    regions: Vec<MemoryRegion>,
    region_map: HashMap<u64, usize>,
    reader: Arc<dyn MemoryReader>,
}

impl MemoryMapping {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            regions: Vec::new(),
            region_map: HashMap::new(),
            reader,
        }
    }

    pub fn add_region(&mut self, region: MemoryRegion) {
        let index = self.regions.len();
        self.region_map.insert(region.start().as_u64(), index);
        self.regions.push(region);
    }

    pub fn add_regions(&mut self, regions: Vec<MemoryRegion>) {
        for region in regions {
            self.add_region(region);
        }
    }

    pub fn get_regions(&self) -> &[MemoryRegion] {
        &self.regions
    }

    pub fn find_region(&self, addr: Address) -> Option<&MemoryRegion> {
        for region in &self.regions {
            if region.contains(addr) {
                return Some(region);
            }
        }
        None
    }

    pub fn find_executable_regions(&self) -> Vec<&MemoryRegion> {
        self.regions.iter().filter(|r| r.is_executable()).collect()
    }

    pub fn find_code_regions(&self) -> Vec<&MemoryRegion> {
        self.regions.iter().filter(|r| r.is_code()).collect()
    }

    pub fn find_data_regions(&self) -> Vec<&MemoryRegion> {
        self.regions.iter().filter(|r| r.is_data()).collect()
    }

    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }

    pub fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        if self.find_region(addr).is_none() {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_bytes(addr, len)
    }
}
