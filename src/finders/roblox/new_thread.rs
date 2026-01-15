// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct NewThreadFinder {
    reader: Arc<dyn MemoryReader>,
}

impl NewThreadFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find(&self, start: Address, end: Address) -> Option<FinderResult> {
        if let Some(result) = self.find_by_pattern(start, end) {
            return Some(result);
        }

        self.find_by_heuristic(start, end)
    }

    fn find_by_pattern(&self, start: Address, end: Address) -> Option<FinderResult> {
        let patterns = vec![
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 F3 ?? ?? A9 ?? ?? ?? ?? 94 ?? ?? ?? F9"),
            Pattern::from_hex("A9 ?? ?? ?? F9 ?? ?? ?? 52 ?? ?? ?? 94 ?? ?? ?? B4"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let addr = current + offset as u64;

                        if self.validate_new_thread(addr) {
                            return Some(FinderResult {
                                name: "NewThread".to_string(),
                                address: addr,
                                confidence: 0.88,
                                method: "pattern".to_string(),
                                category: "roblox".to_string(),
                                signature: Some("lua_State* NewThread(lua_State* L)".to_string()),
                            });
                        }
                    }
                }

                current = current + 4000;
            }
        }

        None
    }

    fn find_by_heuristic(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 128) {
                if self.looks_like_new_thread(&bytes) {
                    let func_start = self.find_function_start(current);

                    if self.validate_new_thread(func_start) {
                        return Some(FinderResult {
                            name: "NewThread".to_string(),
                            address: func_start,
                            confidence: 0.70,
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

    fn validate_new_thread(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 96) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
                return false;
            }

            let mut has_thread_alloc = false;
            let mut has_state_init = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFC000000) == 0x94000000 {
                    has_thread_alloc = true;
                }

                if (insn & 0xFFC00000) == 0xF9000000 {
                    has_state_init = true;
                }
            }

            return has_thread_alloc && has_state_init;
        }

        false
    }

    fn looks_like_new_thread(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 32 {
            return false;
        }

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return false;
        }

        let mut call_count = 0;
        let mut store_count = 0;

        for i in (0..bytes.len().min(64) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFC000000) == 0x94000000 {
                call_count += 1;
            }

            if (insn & 0xFFC00000) == 0xF9000000 {
                store_count += 1;
            }
        }

        call_count >= 1 && store_count >= 2
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

pub fn find_new_thread(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Option<FinderResult> {
    NewThreadFinder::new(reader).find(start, end)
}
