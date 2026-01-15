// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ReferenceAnalyzer {
    reader: Arc<dyn MemoryReader>,
    code_refs: HashMap<u64, Vec<Reference>>,
    data_refs: HashMap<u64, Vec<Reference>>,
    string_refs: HashMap<u64, Vec<Reference>>,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub from: Address,
    pub to: Address,
    pub ref_type: ReferenceType,
    pub instruction_offset: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    Call,
    Jump,
    ConditionalJump,
    DataRead,
    DataWrite,
    StringRef,
    VTableRef,
    PointerRef,
    Unknown,
}

impl ReferenceAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            code_refs: HashMap::new(),
            data_refs: HashMap::new(),
            string_refs: HashMap::new(),
        }
    }

    pub fn analyze_range(&mut self, start: Address, end: Address) -> Result<(), MemoryError> {
        let mut current = start;

        while current < end {
            if let Ok(insn) = self.reader.read_u32(current) {
                self.analyze_instruction(current, insn)?;
            }
            current = current + 4;
        }

        Ok(())
    }

    fn analyze_instruction(&mut self, addr: Address, insn: u32) -> Result<(), MemoryError> {
        if (insn & 0xFC000000) == 0x94000000 {
            let imm26 = (insn & 0x3FFFFFF) as i32;
            let offset = if (imm26 & 0x2000000) != 0 {
                (imm26 | !0x3FFFFFF) << 2
            } else {
                imm26 << 2
            };
            let target = Address::new((addr.as_u64() as i64 + offset as i64) as u64);

            self.add_code_ref(Reference {
                from: addr,
                to: target,
                ref_type: ReferenceType::Call,
                instruction_offset: 0,
            });
        }

        if (insn & 0xFC000000) == 0x14000000 {
            let imm26 = (insn & 0x3FFFFFF) as i32;
            let offset = if (imm26 & 0x2000000) != 0 {
                (imm26 | !0x3FFFFFF) << 2
            } else {
                imm26 << 2
            };
            let target = Address::new((addr.as_u64() as i64 + offset as i64) as u64);

            self.add_code_ref(Reference {
                from: addr,
                to: target,
                ref_type: ReferenceType::Jump,
                instruction_offset: 0,
            });
        }

        if (insn & 0xFF000010) == 0x54000000 {
            let imm19 = ((insn >> 5) & 0x7FFFF) as i32;
            let offset = if (imm19 & 0x40000) != 0 {
                (imm19 | !0x7FFFF) << 2
            } else {
                imm19 << 2
            };
            let target = Address::new((addr.as_u64() as i64 + offset as i64) as u64);

            self.add_code_ref(Reference {
                from: addr,
                to: target,
                ref_type: ReferenceType::ConditionalJump,
                instruction_offset: 0,
            });
        }

        if (insn & 0x9F000000) == 0x90000000 {
            let immlo = (insn >> 29) & 0x3;
            let immhi = (insn >> 5) & 0x7FFFF;
            let imm = ((immhi << 2) | immlo) as i32;
            let imm = if (imm & 0x100000) != 0 {
                imm | !0x1FFFFF
            } else {
                imm
            };

            let op = (insn >> 31) & 1;
            let target = if op == 0 {
                Address::new((addr.as_u64() as i64 + imm as i64) as u64)
            } else {
                let base = (addr.as_u64() as i64) & !0xFFF;
                Address::new((base + ((imm as i64) << 12)) as u64)
            };

            self.add_data_ref(Reference {
                from: addr,
                to: target,
                ref_type: ReferenceType::DataRead,
                instruction_offset: 0,
            });
        }

        Ok(())
    }

    fn add_code_ref(&mut self, ref_: Reference) {
        self.code_refs
            .entry(ref_.to.as_u64())
            .or_insert_with(Vec::new)
            .push(ref_);
    }

    fn add_data_ref(&mut self, ref_: Reference) {
        self.data_refs
            .entry(ref_.to.as_u64())
            .or_insert_with(Vec::new)
            .push(ref_);
    }

    pub fn add_string_ref(&mut self, ref_: Reference) {
        self.string_refs
            .entry(ref_.to.as_u64())
            .or_insert_with(Vec::new)
            .push(ref_);
    }

    pub fn get_code_refs_to(&self, target: Address) -> Vec<&Reference> {
        self.code_refs
            .get(&target.as_u64())
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_data_refs_to(&self, target: Address) -> Vec<&Reference> {
        self.data_refs
            .get(&target.as_u64())
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_string_refs_to(&self, target: Address) -> Vec<&Reference> {
        self.string_refs
            .get(&target.as_u64())
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_all_refs_to(&self, target: Address) -> Vec<&Reference> {
        let mut refs = Vec::new();
        refs.extend(self.get_code_refs_to(target));
        refs.extend(self.get_data_refs_to(target));
        refs.extend(self.get_string_refs_to(target));
        refs
    }

    pub fn get_refs_from(&self, source: Address) -> Vec<Reference> {
        let mut refs = Vec::new();

        for (_, code_refs) in &self.code_refs {
            for ref_ in code_refs {
                if ref_.from == source {
                    refs.push(ref_.clone());
                }
            }
        }

        for (_, data_refs) in &self.data_refs {
            for ref_ in data_refs {
                if ref_.from == source {
                    refs.push(ref_.clone());
                }
            }
        }

        refs
    }

    pub fn count_refs_to(&self, target: Address) -> usize {
        self.get_all_refs_to(target).len()
    }

    pub fn find_callers(&self, target: Address) -> Vec<Address> {
        self.get_code_refs_to(target)
            .into_iter()
            .filter(|r| r.ref_type == ReferenceType::Call)
            .map(|r| r.from)
            .collect()
    }

    pub fn find_string_users(&self, string_addr: Address) -> Vec<Address> {
        self.get_string_refs_to(string_addr)
            .into_iter()
            .map(|r| r.from)
            .collect()
    }

    pub fn get_reference_statistics(&self) -> ReferenceStatistics {
        let mut stats = ReferenceStatistics::default();

        for refs in self.code_refs.values() {
            for ref_ in refs {
                match ref_.ref_type {
                    ReferenceType::Call => stats.call_count += 1,
                    ReferenceType::Jump => stats.jump_count += 1,
                    ReferenceType::ConditionalJump => stats.conditional_jump_count += 1,
                    _ => {}
                }
            }
        }

        stats.data_ref_count = self.data_refs.values().map(|v| v.len()).sum();
        stats.string_ref_count = self.string_refs.values().map(|v| v.len()).sum();
        stats.total_code_targets = self.code_refs.len();
        stats.total_data_targets = self.data_refs.len();

        stats
    }

    pub fn find_most_referenced(&self, top_n: usize) -> Vec<(Address, usize)> {
        let mut ref_counts: HashMap<u64, usize> = HashMap::new();

        for (&addr, refs) in &self.code_refs {
            *ref_counts.entry(addr).or_insert(0) += refs.len();
        }

        let mut sorted: Vec<_> = ref_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(top_n);

        sorted.into_iter().map(|(addr, count)| (Address::new(addr), count)).collect()
    }

    pub fn find_unreferenced_functions(&self, functions: &[Address]) -> Vec<Address> {
        functions
            .iter()
            .filter(|&&func| self.count_refs_to(func) == 0)
            .copied()
            .collect()
    }

    pub fn clear(&mut self) {
        self.code_refs.clear();
        self.data_refs.clear();
        self.string_refs.clear();
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceStatistics {
    pub call_count: usize,
    pub jump_count: usize,
    pub conditional_jump_count: usize,
    pub data_ref_count: usize,
    pub string_ref_count: usize,
    pub total_code_targets: usize,
    pub total_data_targets: usize,
}

impl ReferenceStatistics {
    pub fn total_refs(&self) -> usize {
        self.call_count + self.jump_count + self.conditional_jump_count +
        self.data_ref_count + self.string_ref_count
    }
}

pub fn find_string_references(reader: &dyn MemoryReader, string_addr: Address, code_start: Address, code_end: Address) -> Result<Vec<Address>, MemoryError> {
    let mut refs = Vec::new();
    let target = string_addr.as_u64();
    let mut current = code_start;

    while current < code_end {
        if let Ok(insn) = reader.read_u32(current) {
            if (insn & 0x9F000000) == 0x90000000 {
                let immlo = (insn >> 29) & 0x3;
                let immhi = (insn >> 5) & 0x7FFFF;
                let imm = ((immhi << 2) | immlo) as i32;
                let imm = if (imm & 0x100000) != 0 {
                    imm | !0x1FFFFF
                } else {
                    imm
                };

                let op = (insn >> 31) & 1;
                let resolved = if op == 0 {
                    (current.as_u64() as i64 + imm as i64) as u64
                } else {
                    let base = (current.as_u64() as i64) & !0xFFF;
                    (base + ((imm as i64) << 12)) as u64
                };

                if resolved == target {
                    refs.push(current);
                }
            }
        }
        current = current + 4;
    }

    Ok(refs)
}

pub fn resolve_adrp_add(reader: &dyn MemoryReader, adrp_addr: Address) -> Result<Option<Address>, MemoryError> {
    let adrp_insn = reader.read_u32(adrp_addr)?;

    if (adrp_insn & 0x9F000000) != 0x90000000 || ((adrp_insn >> 31) & 1) != 1 {
        return Ok(None);
    }

    let add_addr = adrp_addr + 4;
    let add_insn = reader.read_u32(add_addr)?;

    if (add_insn & 0xFF000000) != 0x91000000 {
        return Ok(None);
    }

    let immlo = (adrp_insn >> 29) & 0x3;
    let immhi = (adrp_insn >> 5) & 0x7FFFF;
    let imm_page = (((immhi << 2) | immlo) as i32) as i64;
    let imm_page = if (imm_page & 0x100000) != 0 {
        imm_page | !0x1FFFFF
    } else {
        imm_page
    };

    let base = (adrp_addr.as_u64() as i64) & !0xFFF;
    let page_addr = (base + (imm_page << 12)) as u64;

    let imm12 = ((add_insn >> 10) & 0xFFF) as u64;
    let sh = (add_insn >> 22) & 1;
    let offset = if sh == 1 { imm12 << 12 } else { imm12 };

    Ok(Some(Address::new(page_addr + offset)))
}
