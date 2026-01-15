// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryRegion};
use crate::memory::process::ProcessMemory;
use crate::memory::binary::BinaryMemory;
use std::sync::Arc;

pub struct MemoryScanner {
    process_memory: Option<Arc<ProcessMemory>>,
    binary_memory: Option<Arc<BinaryMemory>>,
    regions: Vec<MemoryRegion>,
}

impl MemoryScanner {
    pub fn new() -> Self {
        Self {
            process_memory: None,
            binary_memory: None,
            regions: Vec::new(),
        }
    }

    pub fn with_process(mut self, process: Arc<ProcessMemory>) -> Self {
        self.process_memory = Some(process);
        self
    }

    pub fn with_binary(mut self, binary: Arc<BinaryMemory>) -> Self {
        self.binary_memory = Some(binary);
        self
    }

    pub fn scan_regions(&mut self) -> Result<(), MemoryError> {
        self.regions.clear();

        if let Some(process) = &self.process_memory {
            let process_regions = process.enumerate_regions()?;
            self.regions.extend(process_regions);
        }

        if let Some(binary) = &self.binary_memory {
            let binary_regions = binary.enumerate_regions()?;
            self.regions.extend(binary_regions);
        }

        Ok(())
    }

    pub fn get_regions(&self) -> &[MemoryRegion] {
        &self.regions
    }

    pub fn find_region_by_name(&self, name: &str) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.name() == name)
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

    pub fn find_region_containing(&self, addr: Address) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    pub fn get_process_memory(&self) -> Option<&Arc<ProcessMemory>> {
        self.process_memory.as_ref()
    }

    pub fn get_binary_memory(&self) -> Option<&Arc<BinaryMemory>> {
        self.binary_memory.as_ref()
    }
}

impl Default for MemoryScanner {
    fn default() -> Self {
        Self::new()
    }
}
