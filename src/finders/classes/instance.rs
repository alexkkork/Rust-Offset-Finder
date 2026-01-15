// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::ClassResult;
use std::sync::Arc;

pub struct InstanceClassFinder {
    reader: Arc<dyn MemoryReader>,
}

impl InstanceClassFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<ClassResult> {
        let mut results = Vec::new();

        if let Some(result) = self.find_instance_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_part_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_basepart_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_model_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_workspace_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_datamodel_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_player_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_players_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_script_context_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_localscript_class(start, end) {
            results.push(result);
        }

        if let Some(result) = self.find_modulescript_class(start, end) {
            results.push(result);
        }

        results
    }

    fn find_instance_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Instance";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_confidence(0.90));
        }

        if let Some(addr) = self.find_by_string_ref(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), addr)
                .with_confidence(0.80));
        }

        None
    }

    fn find_part_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Part";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("BasePart")
                .with_confidence(0.88));
        }

        if let Some(addr) = self.find_by_string_ref(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), addr)
                .with_parent("BasePart")
                .with_confidence(0.75));
        }

        None
    }

    fn find_basepart_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "BasePart";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("PVInstance")
                .with_confidence(0.88));
        }

        None
    }

    fn find_model_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Model";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("PVInstance")
                .with_confidence(0.86));
        }

        None
    }

    fn find_workspace_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Workspace";

        if let Some(addr) = self.find_by_string_ref(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), addr)
                .with_parent("Model")
                .with_confidence(0.82));
        }

        None
    }

    fn find_datamodel_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "DataModel";

        if let Some(addr) = self.find_by_string_ref(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), addr)
                .with_parent("ServiceProvider")
                .with_confidence(0.85));
        }

        None
    }

    fn find_player_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Player";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("Instance")
                .with_confidence(0.87));
        }

        None
    }

    fn find_players_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "Players";

        if let Some(addr) = self.find_by_string_ref(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), addr)
                .with_parent("Instance")
                .with_confidence(0.82));
        }

        None
    }

    fn find_script_context_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "ScriptContext";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("Instance")
                .with_confidence(0.85));
        }

        None
    }

    fn find_localscript_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "LocalScript";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("BaseScript")
                .with_confidence(0.84));
        }

        None
    }

    fn find_modulescript_class(&self, start: Address, end: Address) -> Option<ClassResult> {
        let class_name = "ModuleScript";

        if let Some(vtable_addr) = self.find_vtable_by_rtti(class_name, start, end) {
            return Some(ClassResult::new(class_name.to_string(), vtable_addr)
                .with_vtable(vtable_addr)
                .with_parent("LuaSourceContainer")
                .with_confidence(0.84));
        }

        None
    }

    fn find_vtable_by_rtti(&self, class_name: &str, start: Address, end: Address) -> Option<Address> {
        let rtti_patterns = [
            format!("{}@@", class_name),
            format!("_ZTV{}{}", class_name.len(), class_name),
            format!("_ZTI{}{}", class_name.len(), class_name),
        ];

        for rtti in &rtti_patterns {
            if let Some(rtti_addr) = self.find_string(rtti, start, end) {
                if let Some(vtable_addr) = self.find_xref_to_address(rtti_addr, start, end) {
                    return Some(vtable_addr);
                }
            }
        }

        None
    }

    fn find_by_string_ref(&self, class_name: &str, start: Address, end: Address) -> Option<Address> {
        if let Some(string_addr) = self.find_string(class_name, start, end) {
            if let Some(xref_addr) = self.find_xref_to_address(string_addr, start, end) {
                return Some(xref_addr);
            }
        }

        None
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

    fn find_xref_to_address(&self, target: Address, start: Address, end: Address) -> Option<Address> {
        let page = target & !0xFFF;
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
}
