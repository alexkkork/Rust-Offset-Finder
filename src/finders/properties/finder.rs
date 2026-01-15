// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::PropertyResult;
use std::sync::Arc;
use std::collections::HashMap;

pub struct PropertyFinder {
    reader: Arc<dyn MemoryReader>,
}

impl PropertyFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<PropertyResult> {
        let mut results = Vec::new();

        let class_properties = self.get_known_properties();

        for (class_name, properties) in class_properties {
            for (prop_name, prop_type) in properties {
                if let Some(result) = self.find_property(&class_name, &prop_name, &prop_type, start, end) {
                    results.push(result);
                }
            }
        }

        results
    }

    fn get_known_properties(&self) -> HashMap<String, Vec<(String, String)>> {
        let mut map = HashMap::new();

        map.insert("Instance".to_string(), vec![
            ("Name".to_string(), "string".to_string()),
            ("Parent".to_string(), "Instance".to_string()),
            ("ClassName".to_string(), "string".to_string()),
            ("Archivable".to_string(), "bool".to_string()),
        ]);

        map.insert("BasePart".to_string(), vec![
            ("Position".to_string(), "Vector3".to_string()),
            ("Size".to_string(), "Vector3".to_string()),
            ("CFrame".to_string(), "CFrame".to_string()),
            ("Anchored".to_string(), "bool".to_string()),
            ("CanCollide".to_string(), "bool".to_string()),
            ("Transparency".to_string(), "float".to_string()),
            ("Color".to_string(), "Color3".to_string()),
            ("Material".to_string(), "Enum".to_string()),
            ("Velocity".to_string(), "Vector3".to_string()),
            ("RotVelocity".to_string(), "Vector3".to_string()),
            ("Mass".to_string(), "float".to_string()),
            ("AssemblyLinearVelocity".to_string(), "Vector3".to_string()),
            ("AssemblyAngularVelocity".to_string(), "Vector3".to_string()),
        ]);

        map.insert("Humanoid".to_string(), vec![
            ("Health".to_string(), "float".to_string()),
            ("MaxHealth".to_string(), "float".to_string()),
            ("WalkSpeed".to_string(), "float".to_string()),
            ("JumpPower".to_string(), "float".to_string()),
            ("JumpHeight".to_string(), "float".to_string()),
            ("HipHeight".to_string(), "float".to_string()),
            ("DisplayName".to_string(), "string".to_string()),
            ("RigType".to_string(), "Enum".to_string()),
        ]);

        map.insert("Player".to_string(), vec![
            ("Name".to_string(), "string".to_string()),
            ("DisplayName".to_string(), "string".to_string()),
            ("UserId".to_string(), "int64".to_string()),
            ("Character".to_string(), "Model".to_string()),
            ("Team".to_string(), "Team".to_string()),
            ("TeamColor".to_string(), "BrickColor".to_string()),
        ]);

        map.insert("Camera".to_string(), vec![
            ("CFrame".to_string(), "CFrame".to_string()),
            ("Focus".to_string(), "CFrame".to_string()),
            ("FieldOfView".to_string(), "float".to_string()),
            ("ViewportSize".to_string(), "Vector2".to_string()),
            ("CameraType".to_string(), "Enum".to_string()),
            ("CameraSubject".to_string(), "Instance".to_string()),
        ]);

        map.insert("GuiObject".to_string(), vec![
            ("Position".to_string(), "UDim2".to_string()),
            ("Size".to_string(), "UDim2".to_string()),
            ("AnchorPoint".to_string(), "Vector2".to_string()),
            ("Visible".to_string(), "bool".to_string()),
            ("BackgroundColor3".to_string(), "Color3".to_string()),
            ("BackgroundTransparency".to_string(), "float".to_string()),
            ("BorderColor3".to_string(), "Color3".to_string()),
            ("BorderSizePixel".to_string(), "int".to_string()),
            ("ZIndex".to_string(), "int".to_string()),
            ("LayoutOrder".to_string(), "int".to_string()),
            ("Rotation".to_string(), "float".to_string()),
            ("ClipsDescendants".to_string(), "bool".to_string()),
        ]);

        map.insert("TextLabel".to_string(), vec![
            ("Text".to_string(), "string".to_string()),
            ("TextColor3".to_string(), "Color3".to_string()),
            ("TextSize".to_string(), "float".to_string()),
            ("Font".to_string(), "Enum".to_string()),
            ("TextXAlignment".to_string(), "Enum".to_string()),
            ("TextYAlignment".to_string(), "Enum".to_string()),
            ("TextWrapped".to_string(), "bool".to_string()),
            ("TextScaled".to_string(), "bool".to_string()),
        ]);

        map.insert("Sound".to_string(), vec![
            ("SoundId".to_string(), "string".to_string()),
            ("Volume".to_string(), "float".to_string()),
            ("Playing".to_string(), "bool".to_string()),
            ("Looped".to_string(), "bool".to_string()),
            ("TimePosition".to_string(), "float".to_string()),
            ("TimeLength".to_string(), "float".to_string()),
            ("PlaybackSpeed".to_string(), "float".to_string()),
        ]);

        map.insert("Model".to_string(), vec![
            ("PrimaryPart".to_string(), "BasePart".to_string()),
        ]);

        map.insert("RemoteEvent".to_string(), vec![]);
        map.insert("RemoteFunction".to_string(), vec![]);
        map.insert("BindableEvent".to_string(), vec![]);
        map.insert("BindableFunction".to_string(), vec![]);

        map
    }

    fn find_property(&self, class_name: &str, prop_name: &str, prop_type: &str, start: Address, end: Address) -> Option<PropertyResult> {
        if let Some((getter, setter)) = self.find_property_accessors(class_name, prop_name, start, end) {
            return Some(PropertyResult::new(class_name.to_string(), prop_name.to_string())
                .with_getter(getter)
                .with_setter(setter)
                .with_type(prop_type)
                .with_confidence(0.85));
        }

        if let Some(offset) = self.find_property_offset(class_name, prop_name, start, end) {
            return Some(PropertyResult::new(class_name.to_string(), prop_name.to_string())
                .with_offset(offset)
                .with_type(prop_type)
                .with_confidence(0.70));
        }

        None
    }

    fn find_property_accessors(&self, class_name: &str, prop_name: &str, start: Address, end: Address) -> Option<(Address, Address)> {
        let getter_name = format!("get_{}", prop_name);
        let setter_name = format!("set_{}", prop_name);

        let getter = self.find_function_by_name(&getter_name, start, end);
        let setter = self.find_function_by_name(&setter_name, start, end);

        if getter.is_some() || setter.is_some() {
            Some((
                getter.unwrap_or(Address::new(0)),
                setter.unwrap_or(Address::new(0)),
            ))
        } else {
            None
        }
    }

    fn find_property_offset(&self, _class_name: &str, _prop_name: &str, _start: Address, _end: Address) -> Option<u64> {
        None
    }

    fn find_function_by_name(&self, name: &str, start: Address, end: Address) -> Option<Address> {
        if let Some(string_addr) = self.find_string(name, start, end) {
            if let Some(xref_addr) = self.find_xref(string_addr, start, end) {
                return Some(self.find_function_start(xref_addr));
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

    fn find_xref(&self, target: Address, start: Address, end: Address) -> Option<Address> {
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
