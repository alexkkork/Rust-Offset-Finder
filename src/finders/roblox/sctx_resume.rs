// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct SctxResumeFinder {
    reader: Arc<dyn MemoryReader>,
}

impl SctxResumeFinder {
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
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? F9 ?? ?? ?? B9"),
            Pattern::from_hex("F9 ?? ?? ?? B9 ?? ?? ?? 52 ?? ?? ?? 72 ?? ?? ?? 94"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let addr = current + offset as u64;

                        if self.validate_sctx_resume(addr) {
                            return Some(FinderResult {
                                name: "sctx_resume".to_string(),
                                address: addr,
                                confidence: 0.87,
                                method: "pattern".to_string(),
                                category: "roblox".to_string(),
                                signature: Some("int sctxResume(lua_State* L, lua_State* from, int narg)".to_string()),
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
        let search_strings = [
            "cannot resume",
            "script context",
            "resume",
        ];

        for needle in &search_strings {
            if let Some(string_addr) = self.find_string(needle, start, end) {
                if let Some(func_addr) = self.find_xref_to_string(string_addr, start, end) {
                    let func_start = self.find_function_start(func_addr);

                    if self.validate_sctx_resume(func_start) {
                        return Some(FinderResult {
                            name: "sctx_resume".to_string(),
                            address: func_start,
                            confidence: 0.82,
                            method: "string_xref".to_string(),
                            category: "roblox".to_string(),
                            signature: None,
                        });
                    }
                }
            }
        }

        None
    }

    fn find_by_heuristic(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 128) {
                if self.looks_like_sctx_resume(&bytes) {
                    let func_start = self.find_function_start(current);

                    if self.validate_sctx_resume(func_start) {
                        return Some(FinderResult {
                            name: "sctx_resume".to_string(),
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

    fn validate_sctx_resume(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 128) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
                return false;
            }

            let mut has_identity_check = false;
            let mut has_context_access = false;
            let mut has_resume_call = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFFC00000) == 0xB9400000 {
                    has_identity_check = true;
                }

                if (insn & 0xFFC00000) == 0xF9400000 {
                    has_context_access = true;
                }

                if (insn & 0xFC000000) == 0x94000000 {
                    has_resume_call = true;
                }
            }

            return has_identity_check && has_context_access && has_resume_call;
        }

        false
    }

    fn looks_like_sctx_resume(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 32 {
            return false;
        }

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return false;
        }

        let mut call_count = 0;
        let mut load32_count = 0;
        let mut load64_count = 0;

        for i in (0..bytes.len().min(64) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFC000000) == 0x94000000 {
                call_count += 1;
            }

            if (insn & 0xFFC00000) == 0xB9400000 {
                load32_count += 1;
            }

            if (insn & 0xFFC00000) == 0xF9400000 {
                load64_count += 1;
            }
        }

        call_count >= 1 && load32_count >= 1 && load64_count >= 2
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

pub fn find_sctx_resume(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Option<FinderResult> {
    SctxResumeFinder::new(reader).find(start, end)
}
