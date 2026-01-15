// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader, MemoryRegion, MemoryRange, Protection};
use std::ffi::CString;
use libc::{pid_t, c_void, size_t, c_int, c_uint, c_char};

type mach_port_t = c_uint;
type kern_return_t = c_int;
type vm_address_t = u64;
type vm_size_t = u64;
type vm_prot_t = c_int;
type vm_region_flavor_t = c_int;
type vm_region_info_t = *mut c_int;

const KERN_SUCCESS: kern_return_t = 0;
const VM_REGION_BASIC_INFO_64: vm_region_flavor_t = 9;
const VM_REGION_BASIC_INFO_COUNT_64: u32 = 9;
const PROC_ALL_PIDS: u32 = 1;
const PROC_PIDPATHINFO_MAXSIZE: u32 = 4096;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct vm_region_basic_info_64 {
    protection: vm_prot_t,
    max_protection: vm_prot_t,
    inheritance: c_uint,
    shared: c_uint,
    reserved: c_uint,
    offset: u64,
    behavior: c_int,
    user_wired_count: u16,
}

extern "C" {
    fn mach_task_self() -> mach_port_t;
    fn task_for_pid(target_task: mach_port_t, pid: c_int, task: *mut mach_port_t) -> kern_return_t;
    fn vm_read_overwrite(
        target_task: mach_port_t,
        address: vm_address_t,
        size: vm_size_t,
        data: vm_address_t,
        out_size: *mut vm_size_t,
    ) -> kern_return_t;
    fn mach_vm_region(
        target_task: mach_port_t,
        address: *mut vm_address_t,
        size: *mut vm_size_t,
        flavor: vm_region_flavor_t,
        info: vm_region_info_t,
        info_count: *mut u32,
        object_name: *mut mach_port_t,
    ) -> kern_return_t;
    fn proc_listpids(type_: u32, typeinfo: u32, buffer: *mut c_void, buffersize: c_int) -> c_int;
    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
}

pub struct ProcessMemory {
    pid: pid_t,
    task: mach_port_t,
}

impl ProcessMemory {
    pub fn attach(pid: pid_t) -> Result<Self, MemoryError> {
        let mut task: mach_port_t = 0;
        unsafe {
            let self_task = mach_task_self();
            let result = task_for_pid(self_task, pid, &mut task);
            if result != KERN_SUCCESS {
                return Err(MemoryError::ProcessNotFound(format!(
                    "Failed to attach to process {} (error {}). Root privileges may be required.",
                    pid, result
                )));
            }
        }
        Ok(Self { pid, task })
    }

    pub fn attach_by_name(name: &str) -> Result<Self, MemoryError> {
        let pids = Self::find_processes_by_name(name)?;
        if pids.is_empty() {
            return Err(MemoryError::ProcessNotFound(format!("Process '{}' not found", name)));
        }
        Self::attach(pids[0])
    }

    pub fn find_processes_by_name(name: &str) -> Result<Vec<pid_t>, MemoryError> {
        let mut pids = Vec::new();
        let _name_cstr = CString::new(name)
            .map_err(|e| MemoryError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

        let proc_list_size = unsafe { proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
        if proc_list_size <= 0 {
            return Ok(pids);
        }

        let num_pids = proc_list_size as usize / std::mem::size_of::<pid_t>();
        let mut proc_list: Vec<pid_t> = vec![0; num_pids];
        let count = unsafe {
            proc_listpids(
                PROC_ALL_PIDS,
                0,
                proc_list.as_mut_ptr() as *mut c_void,
                proc_list_size,
            )
        };

        if count <= 0 {
            return Ok(pids);
        }

        let actual_count = count as usize / std::mem::size_of::<pid_t>();
        for i in 0..actual_count {
            let pid = proc_list[i];
            if pid == 0 {
                continue;
            }

            let mut path_buffer = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
            let result = unsafe {
                proc_pidpath(pid, path_buffer.as_mut_ptr() as *mut c_void, PROC_PIDPATHINFO_MAXSIZE)
            };

            if result > 0 {
                let path_len = result as usize;
                if let Ok(path_str) = std::str::from_utf8(&path_buffer[..path_len]) {
                    if path_str.contains(name) {
                        pids.push(pid);
                    }
                }
            }
        }

        Ok(pids)
    }

    pub fn pid(&self) -> pid_t {
        self.pid
    }

    pub fn task(&self) -> mach_port_t {
        self.task
    }

    pub fn enumerate_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        let mut regions = Vec::new();
        let mut address: vm_address_t = 0;

        loop {
            let mut size: vm_size_t = 0;
            let mut info: vm_region_basic_info_64 = Default::default();
            let mut info_count: u32 = VM_REGION_BASIC_INFO_COUNT_64;
            let mut object_name: mach_port_t = 0;

            let result = unsafe {
                mach_vm_region(
                    self.task,
                    &mut address,
                    &mut size,
                    VM_REGION_BASIC_INFO_64,
                    &mut info as *mut _ as vm_region_info_t,
                    &mut info_count,
                    &mut object_name,
                )
            };

            if result != KERN_SUCCESS {
                break;
            }

            let protection = Protection::from_flags(info.protection as u32);
            let range = MemoryRange::from_start_size(Address::new(address), size);
            let region = MemoryRegion::new(range, protection, format!("region_{:016x}", address));
            regions.push(region);

            address = address.saturating_add(size);
            if address == 0 || size == 0 {
                break;
            }
        }

        Ok(regions)
    }

    pub fn read_memory(&self, address: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        let mut buffer = vec![0u8; size];
        let mut out_size: vm_size_t = 0;

        let result = unsafe {
            vm_read_overwrite(
                self.task,
                address,
                size as vm_size_t,
                buffer.as_mut_ptr() as vm_address_t,
                &mut out_size,
            )
        };

        if result != KERN_SUCCESS {
            return Err(MemoryError::ReadFailed(address));
        }

        buffer.truncate(out_size as usize);
        Ok(buffer)
    }

    pub fn find_pattern(&self, pattern: &[u8], mask: &[u8], start: u64, end: u64) -> Result<Vec<u64>, MemoryError> {
        let mut results = Vec::new();
        let regions = self.enumerate_regions()?;

        for region in regions {
            let region_start = region.range().start().as_u64();
            let region_end = region.range().end().as_u64();

            if region_end < start || region_start > end {
                continue;
            }

            if !region.protection().is_readable() {
                continue;
            }

            let scan_start = std::cmp::max(region_start, start);
            let scan_end = std::cmp::min(region_end, end);
            let scan_size = (scan_end - scan_start) as usize;

            if scan_size < pattern.len() {
                continue;
            }

            match self.read_memory(scan_start, scan_size) {
                Ok(data) => {
                    for i in 0..=(data.len() - pattern.len()) {
                        let mut matched = true;
                        for j in 0..pattern.len() {
                            if mask[j] != 0 && data[i + j] != pattern[j] {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            results.push(scan_start + i as u64);
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    pub fn scan_for_string(&self, target: &str, start: u64, end: u64) -> Result<Vec<u64>, MemoryError> {
        let pattern = target.as_bytes();
        let mask = vec![1u8; pattern.len()];
        self.find_pattern(pattern, &mask, start, end)
    }

    pub fn get_base_address(&self) -> Result<u64, MemoryError> {
        let regions = self.enumerate_regions()?;
        for region in regions {
            if region.protection().is_executable() {
                return Ok(region.range().start().as_u64());
            }
        }
        Err(MemoryError::ProcessNotFound("No executable region found".to_string()))
    }

    pub fn get_module_base(&self, module_name: &str) -> Result<u64, MemoryError> {
        let regions = self.enumerate_regions()?;
        for region in regions {
            if region.name().contains(module_name) {
                return Ok(region.range().start().as_u64());
            }
        }
        Err(MemoryError::ProcessNotFound(format!("Module '{}' not found", module_name)))
    }

    pub fn read_pointer_chain(&self, base: u64, offsets: &[u64]) -> Result<u64, MemoryError> {
        let mut address = base;
        for (i, &offset) in offsets.iter().enumerate() {
            if i < offsets.len() - 1 {
                address = self.read_u64(Address::new(address + offset))?;
            } else {
                address = address + offset;
            }
        }
        Ok(address)
    }

    pub fn write_memory(&self, _address: u64, _data: &[u8]) -> Result<(), MemoryError> {
        Err(MemoryError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Write operations not supported for safety",
        )))
    }
}

impl MemoryReader for ProcessMemory {
    fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        self.read_memory(addr.as_u64(), len)
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

pub struct ProcessScanner {
    process: ProcessMemory,
    scan_alignment: usize,
    max_scan_size: usize,
}

impl ProcessScanner {
    pub fn new(process: ProcessMemory) -> Self {
        Self {
            process,
            scan_alignment: 1,
            max_scan_size: 0x10000000,
        }
    }

    pub fn with_alignment(mut self, alignment: usize) -> Self {
        self.scan_alignment = alignment;
        self
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_scan_size = max_size;
        self
    }

    pub fn scan_pattern(&self, pattern: &[u8], mask: &[u8]) -> Result<Vec<u64>, MemoryError> {
        let base = self.process.get_base_address()?;
        let end = base + self.max_scan_size as u64;
        self.process.find_pattern(pattern, mask, base, end)
    }

    pub fn scan_string(&self, target: &str) -> Result<Vec<u64>, MemoryError> {
        let base = self.process.get_base_address()?;
        let end = base + self.max_scan_size as u64;
        self.process.scan_for_string(target, base, end)
    }

    pub fn scan_value<T: Copy>(&self, value: T) -> Result<Vec<u64>, MemoryError>
    where
        T: AsRef<[u8]> + Sized,
    {
        let bytes = unsafe {
            std::slice::from_raw_parts(&value as *const T as *const u8, std::mem::size_of::<T>())
        };
        let mask = vec![1u8; bytes.len()];
        let base = self.process.get_base_address()?;
        let end = base + self.max_scan_size as u64;
        self.process.find_pattern(bytes, &mask, base, end)
    }

    pub fn scan_u32(&self, value: u32) -> Result<Vec<u64>, MemoryError> {
        let bytes = value.to_le_bytes();
        let mask = vec![1u8; 4];
        let base = self.process.get_base_address()?;
        let end = base + self.max_scan_size as u64;
        self.process.find_pattern(&bytes, &mask, base, end)
    }

    pub fn scan_u64(&self, value: u64) -> Result<Vec<u64>, MemoryError> {
        let bytes = value.to_le_bytes();
        let mask = vec![1u8; 8];
        let base = self.process.get_base_address()?;
        let end = base + self.max_scan_size as u64;
        self.process.find_pattern(&bytes, &mask, base, end)
    }

    pub fn scan_pointer(&self, address: u64) -> Result<Vec<u64>, MemoryError> {
        self.scan_u64(address)
    }

    pub fn process(&self) -> &ProcessMemory {
        &self.process
    }

    pub fn into_process(self) -> ProcessMemory {
        self.process
    }
}

pub fn find_roblox_process() -> Result<ProcessMemory, MemoryError> {
    let search_names = ["RobloxPlayer", "Roblox", "RobloxStudio"];
    for name in &search_names {
        let pids = ProcessMemory::find_processes_by_name(name)?;
        if !pids.is_empty() {
            return ProcessMemory::attach(pids[0]);
        }
    }
    Err(MemoryError::ProcessNotFound("Roblox process not found".to_string()))
}

pub fn list_all_processes() -> Result<Vec<(pid_t, String)>, MemoryError> {
    let mut processes = Vec::new();

    let proc_list_size = unsafe { proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
    if proc_list_size <= 0 {
        return Ok(processes);
    }

    let num_pids = proc_list_size as usize / std::mem::size_of::<pid_t>();
    let mut proc_list: Vec<pid_t> = vec![0; num_pids];
    let count = unsafe {
        proc_listpids(
            PROC_ALL_PIDS,
            0,
            proc_list.as_mut_ptr() as *mut c_void,
            proc_list_size,
        )
    };

    if count <= 0 {
        return Ok(processes);
    }

    let actual_count = count as usize / std::mem::size_of::<pid_t>();
    for i in 0..actual_count {
        let pid = proc_list[i];
        if pid == 0 {
            continue;
        }

        let mut path_buffer = vec![0u8; PROC_PIDPATHINFO_MAXSIZE as usize];
        let result = unsafe {
            proc_pidpath(pid, path_buffer.as_mut_ptr() as *mut c_void, PROC_PIDPATHINFO_MAXSIZE)
        };

        if result > 0 {
            let path_len = result as usize;
            if let Ok(path_str) = std::str::from_utf8(&path_buffer[..path_len]) {
                let name = std::path::Path::new(path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_str.to_string());
                processes.push((pid, name));
            }
        }
    }

    Ok(processes)
}
