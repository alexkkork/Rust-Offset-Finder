// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct OpcodeLookupFinder {
    reader: Arc<dyn MemoryReader>,
}

impl OpcodeLookupFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find(&self, start: Address, end: Address) -> Option<FinderResult> {
        if let Some(result) = self.find_by_pattern(start, end) {
            return Some(result);
        }

        if let Some(result) = self.find_by_jump_table(start, end) {
            return Some(result);
        }

        self.find_by_heuristic(start, end)
    }

    fn find_by_pattern(&self, start: Address, end: Address) -> Option<FinderResult> {
        let patterns = vec![
            Pattern::from_hex("FD 7B ?? A9 FD ?? ?? 91 ?? ?? ?? 39 71 ?? ?? ?? 54"),
            Pattern::from_hex("39 ?? ?? ?? 51 ?? ?? ?? 71 ?? ?? ?? 54 ?? ?? ?? 10"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let addr = current + offset as u64;

                        if self.validate_opcode_lookup(addr) {
                            return Some(FinderResult {
                                name: "OpcodeLookup".to_string(),
                                address: addr,
                                confidence: 0.90,
                                method: "pattern".to_string(),
                                category: "bytecode".to_string(),
                                signature: Some("void* OpcodeLookup(uint8_t opcode)".to_string()),
                            });
                        }
                    }
                }

                current = current + 4000;
            }
        }

        None
    }

    fn find_by_jump_table(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                if let Some(table_offset) = self.find_potential_jump_table(&bytes) {
                    let table_addr = current + table_offset as u64;

                    if let Some(func_addr) = self.find_function_using_table(table_addr, start, end) {
                        if self.validate_opcode_lookup(func_addr) {
                            return Some(FinderResult {
                                name: "OpcodeLookup".to_string(),
                                address: func_addr,
                                confidence: 0.85,
                                method: "jump_table".to_string(),
                                category: "bytecode".to_string(),
                                signature: None,
                            });
                        }
                    }
                }
            }

            current = current + 4000;
        }

        None
    }

    fn find_by_heuristic(&self, start: Address, end: Address) -> Option<FinderResult> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 256) {
                if self.looks_like_opcode_dispatcher(&bytes) {
                    let func_start = self.find_function_start(current);

                    if self.validate_opcode_lookup(func_start) {
                        return Some(FinderResult {
                            name: "OpcodeLookup".to_string(),
                            address: func_start,
                            confidence: 0.72,
                            method: "heuristic".to_string(),
                            category: "bytecode".to_string(),
                            signature: None,
                        });
                    }
                }
            }

            current = current + 64;
        }

        None
    }

    fn find_potential_jump_table(&self, bytes: &[u8]) -> Option<usize> {
        if bytes.len() < 32 * 4 {
            return None;
        }

        for i in (0..bytes.len() - 128).step_by(8) {
            let mut valid_entries = 0;
            let base_ptr = u64::from_le_bytes([
                bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3],
                bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7],
            ]);

            if base_ptr < 0x100000000 || base_ptr > 0x7FFFFFFFFFFF {
                continue;
            }

            for j in 0..32 {
                let offset = i + j * 8;
                if offset + 8 > bytes.len() {
                    break;
                }

                let ptr = u64::from_le_bytes([
                    bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                    bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
                ]);

                if ptr >= 0x100000000 && ptr <= 0x7FFFFFFFFFFF {
                    let diff = if ptr > base_ptr { ptr - base_ptr } else { base_ptr - ptr };
                    if diff < 0x100000 {
                        valid_entries += 1;
                    }
                }
            }

            if valid_entries >= 16 {
                return Some(i);
            }
        }

        None
    }

    fn find_function_using_table(&self, table_addr: Address, start: Address, end: Address) -> Option<Address> {
        let page = table_addr & !0xFFF;
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                for i in (0..bytes.len() - 8).step_by(4) {
                    let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                    if (insn & 0x9F000000) == 0x90000000 {
                        let immlo = ((insn >> 29) & 0x3) as i64;
                        let immhi = ((insn >> 5) & 0x7FFFF) as i64;
                        let imm = ((immhi << 2) | immlo) << 12;
                        let page_calc = ((current.as_u64() + i as u64) & !0xFFF) as i64 + imm;

                        if page_calc as u64 == page {
                            return Some(self.find_function_start(current + i as u64));
                        }
                    }
                }
            }

            current = current + 4000;
        }

        None
    }

    fn looks_like_opcode_dispatcher(&self, bytes: &[u8]) -> bool {
        if bytes.len() < 64 {
            return false;
        }

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return false;
        }

        let mut cmp_count = 0;
        let mut branch_count = 0;
        let mut load_byte_count = 0;
        let mut adr_count = 0;

        for i in (0..bytes.len().min(128) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0x7F000000) == 0x71000000 {
                cmp_count += 1;
            }

            if (insn & 0xFF000000) == 0x54000000 {
                branch_count += 1;
            }

            if (insn & 0xFF000000) == 0x39000000 || (insn & 0xFFC00000) == 0x39400000 {
                load_byte_count += 1;
            }

            if (insn & 0x9F000000) == 0x10000000 || (insn & 0x9F000000) == 0x90000000 {
                adr_count += 1;
            }
        }

        (cmp_count >= 3 && branch_count >= 3) || (load_byte_count >= 2 && adr_count >= 2)
    }

    fn validate_opcode_lookup(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 192) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
                return false;
            }

            let mut has_byte_load = false;
            let mut has_switch_or_table = false;
            let mut has_indirect_jump = false;

            for i in (0..bytes.len() - 4).step_by(4) {
                let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                if (insn & 0xFFC00000) == 0x39400000 {
                    has_byte_load = true;
                }

                if (insn & 0x7F000000) == 0x71000000 {
                    has_switch_or_table = true;
                }

                if (insn & 0xFFFFFC1F) == 0xD61F0000 || (insn & 0xFFFFFC1F) == 0xD63F0000 {
                    has_indirect_jump = true;
                }

                if (insn & 0x9F000000) == 0x10000000 {
                    has_switch_or_table = true;
                }
            }

            return has_byte_load || (has_switch_or_table && has_indirect_jump);
        }

        false
    }

    fn find_function_start(&self, addr: Address) -> Address {
        let mut current = addr;
        let base = self.reader.get_base_address();

        for _ in 0..512 {
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

pub fn find_opcode_lookup(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Option<FinderResult> {
    OpcodeLookupFinder::new(reader).find(start, end)
}
