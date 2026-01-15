// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;
use std::collections::HashMap;

pub struct DebugInfoAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl DebugInfoAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze_proto_debug(&self, proto_addr: Address) -> Result<ProtoDebugInfo, MemoryError> {
        let mut info = ProtoDebugInfo::new();

        let source_ptr = self.reader.read_u64(proto_addr + 0x60)?;
        if source_ptr != 0 {
            let source_addr = Address::new(source_ptr);
            let len = self.reader.read_u32(source_addr + 0x10)? as usize;
            if len > 0 && len < 0x10000 {
                let data = self.reader.read_bytes(source_addr + 0x18, len)?;
                info.source = String::from_utf8_lossy(&data).to_string();
            }
        }

        let lineinfo_ptr = self.reader.read_u64(proto_addr + 0x38)?;
        let lineinfo_size = self.reader.read_u32(proto_addr + 0x48)? as usize;

        if lineinfo_ptr != 0 && lineinfo_size > 0 {
            info.has_lineinfo = true;
            info.lineinfo_count = lineinfo_size;
        }

        Ok(info)
    }

    pub fn get_line_for_pc(&self, proto_addr: Address, pc: usize) -> Result<Option<u32>, MemoryError> {
        let lineinfo_ptr = self.reader.read_u64(proto_addr + 0x38)?;
        let lineinfo_size = self.reader.read_u32(proto_addr + 0x48)? as usize;

        if lineinfo_ptr == 0 || pc >= lineinfo_size {
            return Ok(None);
        }

        let line = self.reader.read_u8(Address::new(lineinfo_ptr) + pc as u64)?;
        Ok(Some(line as u32))
    }

    pub fn get_local_vars(&self, proto_addr: Address) -> Result<Vec<LocalVarInfo>, MemoryError> {
        let mut locals = Vec::new();

        Ok(locals)
    }

    pub fn get_upvalue_names(&self, proto_addr: Address) -> Result<Vec<String>, MemoryError> {
        let mut names = Vec::new();

        Ok(names)
    }

    pub fn find_function_name(&self, closure_addr: Address) -> Result<Option<String>, MemoryError> {
        Ok(None)
    }

    pub fn get_stack_trace(&self, state_addr: Address) -> Result<Vec<StackFrame>, MemoryError> {
        let mut frames = Vec::new();

        let ci_ptr = self.reader.read_u64(state_addr + 0x28)?;
        let base_ci_ptr = self.reader.read_u64(state_addr + 0x30)?;

        if ci_ptr == 0 || base_ci_ptr == 0 {
            return Ok(frames);
        }

        let mut current_ci = ci_ptr;
        let ci_size = 0x28;

        while current_ci >= base_ci_ptr {
            let func_ptr = self.reader.read_u64(Address::new(current_ci) + 0x08)?;

            if func_ptr != 0 {
                let frame = StackFrame {
                    ci_address: Address::new(current_ci),
                    function_address: Address::new(func_ptr),
                    source: None,
                    line: None,
                    name: None,
                };
                frames.push(frame);
            }

            if current_ci == base_ci_ptr {
                break;
            }
            current_ci = current_ci.saturating_sub(ci_size);
        }

        Ok(frames)
    }

    pub fn format_stack_trace(&self, frames: &[StackFrame]) -> String {
        let mut output = String::new();

        for (idx, frame) in frames.iter().enumerate() {
            output.push_str(&format!("#{} ", idx));

            if let Some(name) = &frame.name {
                output.push_str(name);
            } else {
                output.push_str(&format!("[0x{:X}]", frame.function_address.as_u64()));
            }

            if let Some(source) = &frame.source {
                output.push_str(&format!(" at {}", source));
            }

            if let Some(line) = frame.line {
                output.push_str(&format!(":{}", line));
            }

            output.push('\n');
        }

        output
    }
}

#[derive(Debug, Clone)]
pub struct ProtoDebugInfo {
    pub source: String,
    pub has_lineinfo: bool,
    pub lineinfo_count: usize,
    pub local_vars: Vec<LocalVarInfo>,
    pub upvalue_names: Vec<String>,
    pub linedefined: u32,
    pub lastlinedefined: u32,
}

impl ProtoDebugInfo {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            has_lineinfo: false,
            lineinfo_count: 0,
            local_vars: Vec::new(),
            upvalue_names: Vec::new(),
            linedefined: 0,
            lastlinedefined: 0,
        }
    }

    pub fn has_debug_info(&self) -> bool {
        self.has_lineinfo || !self.source.is_empty()
    }

    pub fn source_name(&self) -> &str {
        if self.source.is_empty() {
            "?"
        } else {
            &self.source
        }
    }
}

impl Default for ProtoDebugInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LocalVarInfo {
    pub name: String,
    pub start_pc: u32,
    pub end_pc: u32,
    pub register: u8,
}

impl LocalVarInfo {
    pub fn is_active_at(&self, pc: u32) -> bool {
        pc >= self.start_pc && pc < self.end_pc
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub ci_address: Address,
    pub function_address: Address,
    pub source: Option<String>,
    pub line: Option<u32>,
    pub name: Option<String>,
}

impl StackFrame {
    pub fn is_lua_frame(&self) -> bool {
        self.source.is_some()
    }

    pub fn is_c_frame(&self) -> bool {
        self.source.is_none()
    }

    pub fn display_location(&self) -> String {
        match (&self.source, self.line) {
            (Some(src), Some(line)) => format!("{}:{}", src, line),
            (Some(src), None) => src.clone(),
            (None, _) => format!("[0x{:X}]", self.function_address.as_u64()),
        }
    }
}

pub struct DebugHooks {
    pub call_hook: Option<Address>,
    pub return_hook: Option<Address>,
    pub line_hook: Option<Address>,
    pub count_hook: Option<Address>,
    pub hook_mask: u8,
    pub hook_count: i32,
}

impl DebugHooks {
    pub fn new() -> Self {
        Self {
            call_hook: None,
            return_hook: None,
            line_hook: None,
            count_hook: None,
            hook_mask: 0,
            hook_count: 0,
        }
    }

    pub fn has_call_hook(&self) -> bool {
        self.hook_mask & 0x01 != 0
    }

    pub fn has_return_hook(&self) -> bool {
        self.hook_mask & 0x02 != 0
    }

    pub fn has_line_hook(&self) -> bool {
        self.hook_mask & 0x04 != 0
    }

    pub fn has_count_hook(&self) -> bool {
        self.hook_mask & 0x08 != 0
    }
}

impl Default for DebugHooks {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BreakpointManager {
    breakpoints: HashMap<(u64, u32), BreakpointInfo>,
    next_id: u32,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_breakpoint(&mut self, proto: Address, pc: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let info = BreakpointInfo {
            id,
            proto_address: proto,
            pc,
            enabled: true,
            hit_count: 0,
            condition: None,
        };

        self.breakpoints.insert((proto.as_u64(), pc), info);
        id
    }

    pub fn remove_breakpoint(&mut self, proto: Address, pc: u32) -> bool {
        self.breakpoints.remove(&(proto.as_u64(), pc)).is_some()
    }

    pub fn get_breakpoint(&self, proto: Address, pc: u32) -> Option<&BreakpointInfo> {
        self.breakpoints.get(&(proto.as_u64(), pc))
    }

    pub fn is_breakpoint_at(&self, proto: Address, pc: u32) -> bool {
        self.breakpoints.contains_key(&(proto.as_u64(), pc))
    }

    pub fn all_breakpoints(&self) -> impl Iterator<Item = &BreakpointInfo> {
        self.breakpoints.values()
    }

    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
    }

    pub fn count(&self) -> usize {
        self.breakpoints.len()
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: u32,
    pub proto_address: Address,
    pub pc: u32,
    pub enabled: bool,
    pub hit_count: u32,
    pub condition: Option<String>,
}

impl BreakpointInfo {
    pub fn should_break(&self) -> bool {
        self.enabled
    }

    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }
}
