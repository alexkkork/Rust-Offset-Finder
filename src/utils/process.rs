// Tue Jan 13 2026 - Alex

use libc::{pid_t, c_int, c_void};

pub struct ProcessUtils;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
}

impl ProcessUtils {
    #[cfg(target_os = "macos")]
    pub fn list_processes() -> Vec<ProcessInfo> {
        let mut processes = Vec::new();

        const PROC_ALL_PIDS: u32 = 1;
        const PROC_PIDPATHINFO_MAXSIZE: usize = 4096;

        extern "C" {
            fn proc_listpids(type_: u32, typeinfo: u32, buffer: *mut c_void, buffersize: c_int) -> c_int;
            fn proc_pidpath(pid: pid_t, buffer: *mut c_void, buffersize: u32) -> c_int;
            fn proc_name(pid: pid_t, buffer: *mut c_void, buffersize: u32) -> c_int;
        }

        let mut pids: Vec<pid_t> = vec![0; 4096];
        let pids_ptr = pids.as_mut_ptr() as *mut c_void;
        let pids_size = (pids.len() * std::mem::size_of::<pid_t>()) as c_int;

        let count = unsafe { proc_listpids(PROC_ALL_PIDS, 0, pids_ptr, pids_size) };
        if count <= 0 {
            return processes;
        }

        let pid_count = count as usize / std::mem::size_of::<pid_t>();

        for i in 0..pid_count {
            let pid = pids[i];
            if pid == 0 {
                continue;
            }

            let mut name_buf = vec![0u8; 256];
            let name_result = unsafe {
                proc_name(pid, name_buf.as_mut_ptr() as *mut c_void, name_buf.len() as u32)
            };

            let name = if name_result > 0 {
                let len = name_buf.iter().position(|&c| c == 0).unwrap_or(name_buf.len());
                String::from_utf8_lossy(&name_buf[..len]).to_string()
            } else {
                continue;
            };

            let mut path_buf = vec![0u8; PROC_PIDPATHINFO_MAXSIZE];
            let path_result = unsafe {
                proc_pidpath(pid, path_buf.as_mut_ptr() as *mut c_void, path_buf.len() as u32)
            };

            let path = if path_result > 0 {
                let len = path_buf.iter().position(|&c| c == 0).unwrap_or(path_buf.len());
                Some(String::from_utf8_lossy(&path_buf[..len]).to_string())
            } else {
                None
            };

            processes.push(ProcessInfo {
                pid: pid as u32,
                name,
                path,
            });
        }

        processes
    }

    #[cfg(not(target_os = "macos"))]
    pub fn list_processes() -> Vec<ProcessInfo> {
        Vec::new()
    }

    pub fn find_process_by_name(name: &str) -> Option<ProcessInfo> {
        let processes = Self::list_processes();
        let name_lower = name.to_lowercase();

        processes.into_iter()
            .find(|p| p.name.to_lowercase().contains(&name_lower))
    }

    pub fn find_processes_by_name(name: &str) -> Vec<ProcessInfo> {
        let processes = Self::list_processes();
        let name_lower = name.to_lowercase();

        processes.into_iter()
            .filter(|p| p.name.to_lowercase().contains(&name_lower))
            .collect()
    }

    pub fn find_roblox_process() -> Option<ProcessInfo> {
        let roblox_names = ["RobloxPlayer", "RobloxStudio", "Roblox"];

        for name in &roblox_names {
            if let Some(process) = Self::find_process_by_name(name) {
                return Some(process);
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    pub fn get_process_path(pid: u32) -> Option<String> {
        const PROC_PIDPATHINFO_MAXSIZE: usize = 4096;

        extern "C" {
            fn proc_pidpath(pid: pid_t, buffer: *mut c_void, buffersize: u32) -> c_int;
        }

        let mut path_buf = vec![0u8; PROC_PIDPATHINFO_MAXSIZE];
        let result = unsafe {
            proc_pidpath(pid as pid_t, path_buf.as_mut_ptr() as *mut c_void, path_buf.len() as u32)
        };

        if result > 0 {
            let len = path_buf.iter().position(|&c| c == 0).unwrap_or(path_buf.len());
            Some(String::from_utf8_lossy(&path_buf[..len]).to_string())
        } else {
            None
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn get_process_path(_pid: u32) -> Option<String> {
        None
    }

    pub fn is_process_running(pid: u32) -> bool {
        unsafe {
            libc::kill(pid as pid_t, 0) == 0
        }
    }

    pub fn get_current_pid() -> u32 {
        unsafe { libc::getpid() as u32 }
    }

    pub fn get_current_uid() -> u32 {
        unsafe { libc::getuid() }
    }

    pub fn is_root() -> bool {
        Self::get_current_uid() == 0
    }
}

impl ProcessInfo {
    pub fn is_roblox(&self) -> bool {
        let name_lower = self.name.to_lowercase();
        name_lower.contains("roblox")
    }
}

pub fn find_roblox() -> Option<ProcessInfo> {
    ProcessUtils::find_roblox_process()
}

pub fn list_all() -> Vec<ProcessInfo> {
    ProcessUtils::list_processes()
}

pub fn find_by_name(name: &str) -> Option<ProcessInfo> {
    ProcessUtils::find_process_by_name(name)
}

pub fn is_running(pid: u32) -> bool {
    ProcessUtils::is_process_running(pid)
}
