// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader, MemoryRegion, MemoryRange, Protection};
use goblin::mach::{Mach, MachO};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct BinaryMemory {
    data: Arc<Vec<u8>>,
    base_address: Address,
    path: PathBuf,
    text_offset: u64,
    text_size: u64,
    data_offset: u64,
    data_size: u64,
}

#[derive(Debug, Clone)]
pub struct BinarySegment {
    pub name: String,
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
    pub protection: Protection,
}

#[derive(Debug, Clone)]
pub struct BinarySection {
    pub segname: String,
    pub sectname: String,
    pub addr: u64,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct BinarySymbol {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub is_external: bool,
}

impl BinaryMemory {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, MemoryError> {
        let path_buf = path.as_ref().to_path_buf();
        let mut file = File::open(path.as_ref()).map_err(MemoryError::Io)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(MemoryError::Io)?;

        let (text_offset, text_size, data_offset, data_size) = Self::parse_segments(&data)?;
        let base_address = Address::new(0x100000000);

        Ok(Self {
            data: Arc::new(data),
            base_address,
            path: path_buf,
            text_offset,
            text_size,
            data_offset,
            data_size,
        })
    }

    fn parse_segments(data: &[u8]) -> Result<(u64, u64, u64, u64), MemoryError> {
        let mach = Mach::parse(data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut text_offset = 0u64;
        let mut text_size = 0u64;
        let mut data_offset = 0u64;
        let mut data_size = 0u64;

        for segment in &macho.segments {
            let segname = std::str::from_utf8(&segment.segname)
                .unwrap_or("")
                .trim_end_matches('\0');
            if segname == "__TEXT" {
                text_offset = segment.fileoff;
                text_size = segment.filesize;
            } else if segname == "__DATA" {
                data_offset = segment.fileoff;
                data_size = segment.filesize;
            }
        }

        Ok((text_offset, text_size, data_offset, data_size))
    }

    pub fn enumerate_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut regions = Vec::new();
        for segment in &macho.segments {
            let segname = std::str::from_utf8(&segment.segname)
                .unwrap_or("")
                .trim_end_matches('\0');
            let protection = if segname == "__TEXT" {
                Protection::ReadExecute
            } else if segname == "__DATA" {
                Protection::ReadWrite
            } else {
                Protection::Read
            };

            let range =
                MemoryRange::from_start_size(Address::new(segment.vmaddr), segment.vmsize);
            let region = MemoryRegion::new(range, protection, segname.to_string());
            regions.push(region);
        }

        Ok(regions)
    }

    pub fn get_segments(&self) -> Result<Vec<BinarySegment>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut segments = Vec::new();
        for segment in &macho.segments {
            let segname = std::str::from_utf8(&segment.segname)
                .unwrap_or("")
                .trim_end_matches('\0');
            let protection = Protection::from_flags(segment.initprot);

            segments.push(BinarySegment {
                name: segname.to_string(),
                vmaddr: segment.vmaddr,
                vmsize: segment.vmsize,
                fileoff: segment.fileoff,
                filesize: segment.filesize,
                protection,
            });
        }

        Ok(segments)
    }

    pub fn get_sections(&self) -> Result<Vec<BinarySection>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut sections = Vec::new();
        for segment in &macho.segments {
            let segname = std::str::from_utf8(&segment.segname)
                .unwrap_or("")
                .trim_end_matches('\0');

            for section_result in segment.into_iter() {
                if let Ok((section, _data)) = section_result {
                    let sectname = std::str::from_utf8(&section.sectname)
                        .unwrap_or("")
                        .trim_end_matches('\0');
                    sections.push(BinarySection {
                        segname: segname.to_string(),
                        sectname: sectname.to_string(),
                        addr: section.addr,
                        size: section.size,
                        offset: section.offset as u64,
                    });
                }
            }
        }

        Ok(sections)
    }

    pub fn get_symbols(&self) -> Result<Vec<BinarySymbol>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut symbols = Vec::new();
        for sym in macho.symbols() {
            if let Ok((name, nlist)) = sym {
                symbols.push(BinarySymbol {
                    name: name.to_string(),
                    address: nlist.n_value,
                    size: 0,
                    is_external: nlist.is_global(),
                });
            }
        }

        Ok(symbols)
    }

    pub fn get_exports(&self) -> Result<Vec<BinarySymbol>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut exports = Vec::new();
        if let Ok(export_list) = macho.exports() {
            for export in export_list {
                exports.push(BinarySymbol {
                    name: export.name,
                    address: export.offset,
                    size: export.size as u64,
                    is_external: true,
                });
            }
        }

        Ok(exports)
    }

    pub fn get_imports(&self) -> Result<Vec<String>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        let mut imports = Vec::new();
        if let Ok(import_list) = macho.imports() {
            for import in import_list {
                imports.push(import.name.to_string());
            }
        }

        Ok(imports)
    }

    pub fn base_address(&self) -> Address {
        self.base_address
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn get_section_data(&self, segname: &str, sectname: &str) -> Option<Vec<u8>> {
        let sections = self.get_sections().ok()?;
        for section in sections {
            if section.segname == segname && section.sectname == sectname {
                let start = section.offset as usize;
                let end = start + section.size as usize;
                if end <= self.data.len() {
                    return Some(self.data[start..end].to_vec());
                }
            }
        }
        None
    }

    pub fn get_text_section(&self) -> Option<Vec<u8>> {
        self.get_section_data("__TEXT", "__text")
    }

    pub fn get_data_section(&self) -> Option<Vec<u8>> {
        self.get_section_data("__DATA", "__data")
    }

    pub fn get_cstring_section(&self) -> Option<Vec<u8>> {
        self.get_section_data("__TEXT", "__cstring")
    }

    pub fn find_symbol(&self, name: &str) -> Option<u64> {
        let symbols = self.get_symbols().ok()?;
        for sym in symbols {
            if sym.name == name || sym.name == format!("_{}", name) {
                return Some(sym.address);
            }
        }
        None
    }

    pub fn find_string(&self, target: &str) -> Vec<u64> {
        let mut results = Vec::new();
        let target_bytes = target.as_bytes();
        let data = &self.data;

        for i in 0..data.len().saturating_sub(target_bytes.len()) {
            if &data[i..i + target_bytes.len()] == target_bytes {
                results.push(self.base_address.as_u64() + i as u64);
            }
        }

        results
    }

    pub fn find_pattern(&self, pattern: &[u8], mask: &[u8]) -> Vec<u64> {
        let mut results = Vec::new();
        let data = &self.data;

        if pattern.len() != mask.len() || pattern.is_empty() {
            return results;
        }

        for i in 0..data.len().saturating_sub(pattern.len()) {
            let mut matched = true;
            for j in 0..pattern.len() {
                if mask[j] != 0 && data[i + j] != pattern[j] {
                    matched = false;
                    break;
                }
            }
            if matched {
                results.push(self.base_address.as_u64() + i as u64);
            }
        }

        results
    }

    pub fn find_xrefs(&self, target: u64) -> Vec<u64> {
        let mut results = Vec::new();
        let target_bytes = target.to_le_bytes();
        let data = &self.data;

        for i in 0..data.len().saturating_sub(8) {
            if &data[i..i + 8] == &target_bytes {
                results.push(self.base_address.as_u64() + i as u64);
            }
        }

        results
    }

    pub fn file_offset_to_virtual(&self, offset: u64) -> Option<u64> {
        let segments = self.get_segments().ok()?;
        for seg in segments {
            if offset >= seg.fileoff && offset < seg.fileoff + seg.filesize {
                return Some(seg.vmaddr + (offset - seg.fileoff));
            }
        }
        None
    }

    pub fn virtual_to_file_offset(&self, addr: u64) -> Option<u64> {
        let segments = self.get_segments().ok()?;
        for seg in segments {
            if addr >= seg.vmaddr && addr < seg.vmaddr + seg.vmsize {
                return Some(seg.fileoff + (addr - seg.vmaddr));
            }
        }
        None
    }

    pub fn read_at_offset(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset + len <= self.data.len() {
            Some(&self.data[offset..offset + len])
        } else {
            None
        }
    }

    pub fn entry_point(&self) -> Result<u64, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        Ok(macho.entry)
    }

    pub fn is_64bit(&self) -> Result<bool, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        Ok(macho.is_64)
    }

    pub fn is_arm64(&self) -> Result<bool, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        Ok(macho.header.cputype() == goblin::mach::cputype::CPU_TYPE_ARM64)
    }

    pub fn libraries(&self) -> Result<Vec<String>, MemoryError> {
        let mach = Mach::parse(&self.data)
            .map_err(|e| MemoryError::BinaryParseError(format!("Failed to parse Mach-O: {}", e)))?;

        let macho = match mach {
            Mach::Binary(m) => m,
            Mach::Fat(_) => {
                return Err(MemoryError::BinaryParseError(
                    "Fat binaries not supported".to_string(),
                ))
            }
        };

        Ok(macho.libs.iter().map(|s| s.to_string()).collect())
    }
}

impl MemoryReader for BinaryMemory {
    fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        let virtual_addr = addr.as_u64();
        let file_offset = self
            .virtual_to_file_offset(virtual_addr)
            .ok_or_else(|| MemoryError::OutOfBounds(virtual_addr))?;

        let offset = file_offset as usize;
        if offset + len > self.data.len() {
            return Err(MemoryError::OutOfBounds(virtual_addr));
        }
        Ok(self.data[offset..offset + len].to_vec())
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

pub struct BinaryScanner {
    binary: BinaryMemory,
}

impl BinaryScanner {
    pub fn new(binary: BinaryMemory) -> Self {
        Self { binary }
    }

    pub fn scan_pattern(&self, pattern: &[u8], mask: &[u8]) -> Vec<u64> {
        self.binary.find_pattern(pattern, mask)
    }

    pub fn scan_string(&self, target: &str) -> Vec<u64> {
        self.binary.find_string(target)
    }

    pub fn scan_xrefs(&self, target: u64) -> Vec<u64> {
        self.binary.find_xrefs(target)
    }

    pub fn binary(&self) -> &BinaryMemory {
        &self.binary
    }

    pub fn into_binary(self) -> BinaryMemory {
        self.binary
    }
}

pub fn load_roblox_binary<P: AsRef<Path>>(path: P) -> Result<BinaryMemory, MemoryError> {
    let binary = BinaryMemory::load(path)?;
    if !binary.is_arm64()? {
        return Err(MemoryError::BinaryParseError(
            "Binary is not ARM64".to_string(),
        ));
    }
    Ok(binary)
}
