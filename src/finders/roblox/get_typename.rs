// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct GetTypenameFinder {
    reader: Arc<dyn MemoryReader>,
}

impl GetTypenameFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find(&self, start: Address, end: Address) -> Option<FinderResult> {
        if let Some(result) = self.find_by_pattern(start, end) {
            return Some(result);
        }

        if let Some(result) = self.find_by_string_ref(start, end) {
            return Some(result);
        }

        self.find_by_heuristic(start, end)
    }

    fn find_by_pattern(&self, start: Address, end: Address) -> Option<FinderResult> {
        let patterns = vec![
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 39 71 ?? ?? ?? 54"),
            Pattern::from_hex("39 ?? ?? ?? 71 ?? ?? ?? 54 ?? ?? ?? 90 ?? ?? ?? 91"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let addr = current + offset as u64;

                        if self.validate_get_typename(addr) {
                            return Some(FinderResult {
                                name: "GetTypename".to_string(),
                                address: addr,
                                confidence: 0.86,
                                method: "pattern".to_string(),
                                category: "roblox".to_string(),
                                signature: Some("const char* GetTypename(lua_State* L, int index)".to_string()),
                            });
                        }
                    }
                }

                current = current + 4000;
            }
        }

        None
    }

    fn find_by_string_ref(&self, start: Address, end: Address) -> Option<FinderResult> {
        let type_strings = [
            "nil", "boolean", "userdata", "number",
            "string", "table", "function", "thread",
        ];

        let mut found_count = 0;
        let mut potential_func = Address::new(0);

        for type_str in &type_strings {
            if let Some(string_addr) = self.find_string(type_str, start, end) {
                if let Some(func_addr) = self.find_xref_to_string(string_addr, start, end) {
                    let func_start = self.find_function_start(func_addr);

                    if found_count == 0 || potential_func == func_start {
                        potential_func = func_start;
                        found_count += 1;
                    }
                }
            }
        }

        if found_count >= 3 {
            if self.validate_get_typename(potential_func) {
                return Some(FinderResult {
                    name: "GetTypename".to_string(),
                    address: potential_func,
                    confidence: 0.85,
                    method: "string_xref".to_string(),
                    category: "roblox".to_string(),
                    signature: None,
                });
            }
        }

        None
    }

    fn find_by_heuristic(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 128) {
                if self.looks_like_get_typename(&bytes) {
                    let func_start = self.find_function_start(current);

                    if self.validate_get_typename(func_start) {
                        return Some(FinderResult {
                            name: "GetTypename".to_string(),
                            address: func_start,
                            confidence: 0.65,
                            method: "heuristic".to_string(),
                            category: "roblox".to_string(),
                            signature: None,
                        });
                    }
                }
            }

            current = current + 64;
        }

        None
    }

    fn validate_get_typename(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 96) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
                return false;
            }

            let mut has_type_load = false;
            let mut has_switch_pattern = false;
            let mut has_string_return = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFF000000) == 0x39000000 {
                    has_type_load = true;
                }

                if (insn & 0x7F000000) == 0x71000000 {
                    has_switch_pattern = true;
                }

                if (insn & 0x9F000000) == 0x90000000 {
                    has_string_return = true;
                }
            }

            return has_type_load && (has_switch_pattern || has_string_return);
        }

        false
    }

    fn looks_like_get_typename(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 32 {
            return false;
        }

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return false;
        }

        let mut cmp_count = 0;
        let mut branch_count = 0;

        for i in (0..bytes.len().min(80) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0x7F000000) == 0x71000000 {
                cmp_count += 1;
            }

            if (insn & 0xFF000000) == 0x54000000 {
                branch_count += 1;
            }
        }

        cmp_count >= 2 && branch_count >= 2
    }

    fn find_string(&self, needle: &str, start: Address, end: Address) -> Option<Address> {
        let needle_bytes = needle.as_bytes();
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                if let Some(pos) = bytes.windows(needle_bytes.len())
                    .position(|w| w == needle_bytes)
                {
                    return Some(current + pos as u64);
                }
            }

            current = current + 4000;
        }

        None
    }

    fn find_xref_to_string(&self, string_addr: Address, start: Address, end: Address) -> Option<Address> {
        let page = string_addr & !0xFFF;

        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                for i in (0..bytes.len() - 4).step_by(4) {
                    let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                    if (insn & 0x9F000000) == 0x90000000 {
                        let immlo = ((insn >> 29) & 0x3) as i64;
                        let immhi = ((insn >> 5) & 0x7FFFF) as i64;
                        let imm = ((immhi << 2) | immlo) << 12;
                        let page_calc = ((current.as_u64() + i as u64) & !0xFFF) as i64 + imm;

                        if page_calc as u64 == page {
                            return Some(current + i as u64);
                        }
                    }
                }
            }

            current = current + 4000;
        }

        None
    }

    fn find_function_start(&self, addr: Address) -> Address {
        let mut current = addr;
        let base = self.reader.get_base_address();

        for _ in 0..256 {
            if current <= base {
                break;
            }

            if let Ok(bytes) = self.reader.read_bytes(current, 4) {
                let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                if (insn & 0x7F800000) == 0x29000000 || (insn & 0x7F800000) == 0x6D000000 {
                    return current;
                }

                if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                    return current + 4;
                }
            }

            current = current - 4;
        }

        addr
    }
}

pub fn find_get_typename(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Option<FinderResult> {
    GetTypenameFinder::new(reader).find(start, end)
}
