// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::MethodResult;
use crate::finders::classes::vtable::VTableAnalyzer;
use std::sync::Arc;
use std::collections::HashMap;

pub struct MethodFinder {
    reader: Arc<dyn MemoryReader>,
    vtable_analyzer: VTableAnalyzer,
}

impl MethodFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            vtable_analyzer: VTableAnalyzer::new(reader.clone()),
            reader,
        }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<MethodResult> {
        let mut results = Vec::new();

        let class_methods = self.get_known_methods();

        for (class_name, methods) in class_methods {
            for (method_name, signature) in methods {
                if let Some(result) = self.find_method(&class_name, &method_name, &signature, start, end) {
                    results.push(result);
                }
            }
        }

        results
    }

    fn get_known_methods(&self) -> HashMap<String, Vec<(String, String)>> {
        let mut map = HashMap::new();

        map.insert("Instance".to_string(), vec![
            ("FindFirstChild".to_string(), "Instance* FindFirstChild(const char* name, bool recursive)".to_string()),
            ("FindFirstChildOfClass".to_string(), "Instance* FindFirstChildOfClass(const char* className)".to_string()),
            ("FindFirstChildWhichIsA".to_string(), "Instance* FindFirstChildWhichIsA(const char* className, bool recursive)".to_string()),
            ("FindFirstAncestor".to_string(), "Instance* FindFirstAncestor(const char* name)".to_string()),
            ("FindFirstAncestorOfClass".to_string(), "Instance* FindFirstAncestorOfClass(const char* className)".to_string()),
            ("FindFirstAncestorWhichIsA".to_string(), "Instance* FindFirstAncestorWhichIsA(const char* className)".to_string()),
            ("GetChildren".to_string(), "Array<Instance*> GetChildren()".to_string()),
            ("GetDescendants".to_string(), "Array<Instance*> GetDescendants()".to_string()),
            ("IsA".to_string(), "bool IsA(const char* className)".to_string()),
            ("IsAncestorOf".to_string(), "bool IsAncestorOf(Instance* descendant)".to_string()),
            ("IsDescendantOf".to_string(), "bool IsDescendantOf(Instance* ancestor)".to_string()),
            ("Destroy".to_string(), "void Destroy()".to_string()),
            ("Clone".to_string(), "Instance* Clone()".to_string()),
            ("GetFullName".to_string(), "std::string GetFullName()".to_string()),
            ("WaitForChild".to_string(), "Instance* WaitForChild(const char* name, double timeout)".to_string()),
            ("GetPropertyChangedSignal".to_string(), "RBXScriptSignal GetPropertyChangedSignal(const char* property)".to_string()),
            ("SetAttribute".to_string(), "void SetAttribute(const char* name, Variant value)".to_string()),
            ("GetAttribute".to_string(), "Variant GetAttribute(const char* name)".to_string()),
            ("GetAttributes".to_string(), "Dictionary GetAttributes()".to_string()),
        ]);

        map.insert("BasePart".to_string(), vec![
            ("GetMass".to_string(), "float GetMass()".to_string()),
            ("GetTouchingParts".to_string(), "Array<BasePart*> GetTouchingParts()".to_string()),
            ("GetConnectedParts".to_string(), "Array<BasePart*> GetConnectedParts(bool recursive)".to_string()),
            ("CanCollideWith".to_string(), "bool CanCollideWith(BasePart* other)".to_string()),
            ("ApplyImpulse".to_string(), "void ApplyImpulse(Vector3 impulse)".to_string()),
            ("ApplyImpulseAtPosition".to_string(), "void ApplyImpulseAtPosition(Vector3 impulse, Vector3 position)".to_string()),
            ("ApplyAngularImpulse".to_string(), "void ApplyAngularImpulse(Vector3 impulse)".to_string()),
            ("BreakJoints".to_string(), "void BreakJoints()".to_string()),
            ("MakeJoints".to_string(), "void MakeJoints()".to_string()),
            ("Resize".to_string(), "bool Resize(NormalId normalId, int deltaAmount)".to_string()),
            ("GetRootPart".to_string(), "BasePart* GetRootPart()".to_string()),
        ]);

        map.insert("Model".to_string(), vec![
            ("GetPrimaryPartCFrame".to_string(), "CFrame GetPrimaryPartCFrame()".to_string()),
            ("SetPrimaryPartCFrame".to_string(), "void SetPrimaryPartCFrame(CFrame cframe)".to_string()),
            ("GetBoundingBox".to_string(), "void GetBoundingBox(CFrame& orientation, Vector3& size)".to_string()),
            ("TranslateBy".to_string(), "void TranslateBy(Vector3 delta)".to_string()),
            ("MoveTo".to_string(), "void MoveTo(Vector3 position)".to_string()),
            ("GetExtentsSize".to_string(), "Vector3 GetExtentsSize()".to_string()),
        ]);

        map.insert("Humanoid".to_string(), vec![
            ("MoveTo".to_string(), "void MoveTo(Vector3 location, BasePart* part)".to_string()),
            ("Move".to_string(), "void Move(Vector3 moveDirection, bool relativeToCamera)".to_string()),
            ("TakeDamage".to_string(), "void TakeDamage(float amount)".to_string()),
            ("LoadAnimation".to_string(), "AnimationTrack* LoadAnimation(Animation* animation)".to_string()),
            ("EquipTool".to_string(), "void EquipTool(Tool* tool)".to_string()),
            ("UnequipTools".to_string(), "void UnequipTools()".to_string()),
            ("GetState".to_string(), "HumanoidStateType GetState()".to_string()),
            ("ChangeState".to_string(), "void ChangeState(HumanoidStateType state)".to_string()),
            ("AddAccessory".to_string(), "void AddAccessory(Accessory* accessory)".to_string()),
            ("GetAccessories".to_string(), "Array<Accessory*> GetAccessories()".to_string()),
            ("ApplyDescription".to_string(), "void ApplyDescription(HumanoidDescription* description)".to_string()),
            ("GetAppliedDescription".to_string(), "HumanoidDescription* GetAppliedDescription()".to_string()),
        ]);

        map.insert("Player".to_string(), vec![
            ("Kick".to_string(), "void Kick(const char* message)".to_string()),
            ("GetMouse".to_string(), "Mouse* GetMouse()".to_string()),
            ("LoadCharacter".to_string(), "void LoadCharacter()".to_string()),
            ("LoadCharacterWithHumanoidDescription".to_string(), "void LoadCharacterWithHumanoidDescription(HumanoidDescription* description)".to_string()),
            ("RequestStreamAroundAsync".to_string(), "void RequestStreamAroundAsync(Vector3 position)".to_string()),
            ("GetFriendsOnline".to_string(), "Array<Dictionary> GetFriendsOnline(int maxFriends)".to_string()),
            ("GetRankInGroup".to_string(), "int GetRankInGroup(int groupId)".to_string()),
            ("GetRoleInGroup".to_string(), "std::string GetRoleInGroup(int groupId)".to_string()),
            ("IsFriendsWith".to_string(), "bool IsFriendsWith(int64 userId)".to_string()),
            ("IsInGroup".to_string(), "bool IsInGroup(int groupId)".to_string()),
        ]);

        map.insert("Camera".to_string(), vec![
            ("ScreenPointToRay".to_string(), "Ray ScreenPointToRay(float x, float y, float depth)".to_string()),
            ("ViewportPointToRay".to_string(), "Ray ViewportPointToRay(float x, float y, float depth)".to_string()),
            ("WorldToScreenPoint".to_string(), "Vector3 WorldToScreenPoint(Vector3 worldPoint)".to_string()),
            ("WorldToViewportPoint".to_string(), "Vector3 WorldToViewportPoint(Vector3 worldPoint)".to_string()),
            ("GetPartsObscuringTarget".to_string(), "Array<BasePart*> GetPartsObscuringTarget(Array<Vector3> castPoints, Array<Instance*> ignoreList)".to_string()),
            ("Interpolate".to_string(), "void Interpolate(CFrame endCFrame, CFrame endFocus, float duration)".to_string()),
            ("SetRoll".to_string(), "void SetRoll(float rollAngle)".to_string()),
            ("GetRoll".to_string(), "float GetRoll()".to_string()),
        ]);

        map.insert("DataModel".to_string(), vec![
            ("GetService".to_string(), "Instance* GetService(const char* serviceName)".to_string()),
            ("FindService".to_string(), "Instance* FindService(const char* serviceName)".to_string()),
            ("BindToClose".to_string(), "void BindToClose(Function callback)".to_string()),
        ]);

        map.insert("TweenService".to_string(), vec![
            ("Create".to_string(), "Tween* Create(Instance* instance, TweenInfo info, Dictionary goals)".to_string()),
            ("GetValue".to_string(), "Variant GetValue(float alpha, EasingStyle style, EasingDirection direction)".to_string()),
        ]);

        map.insert("RunService".to_string(), vec![
            ("IsClient".to_string(), "bool IsClient()".to_string()),
            ("IsServer".to_string(), "bool IsServer()".to_string()),
            ("IsStudio".to_string(), "bool IsStudio()".to_string()),
            ("IsEdit".to_string(), "bool IsEdit()".to_string()),
            ("IsRunning".to_string(), "bool IsRunning()".to_string()),
            ("BindToRenderStep".to_string(), "void BindToRenderStep(const char* name, int priority, Function callback)".to_string()),
            ("UnbindFromRenderStep".to_string(), "void UnbindFromRenderStep(const char* name)".to_string()),
        ]);

        map.insert("UserInputService".to_string(), vec![
            ("IsKeyDown".to_string(), "bool IsKeyDown(KeyCode keyCode)".to_string()),
            ("IsMouseButtonPressed".to_string(), "bool IsMouseButtonPressed(UserInputType button)".to_string()),
            ("GetKeysPressed".to_string(), "Array<InputObject> GetKeysPressed()".to_string()),
            ("GetMouseButtonsPressed".to_string(), "Array<InputObject> GetMouseButtonsPressed()".to_string()),
            ("GetMouseLocation".to_string(), "Vector2 GetMouseLocation()".to_string()),
            ("GetFocusedTextBox".to_string(), "TextBox* GetFocusedTextBox()".to_string()),
            ("IsGamepadButtonDown".to_string(), "bool IsGamepadButtonDown(UserInputType gamepad, KeyCode button)".to_string()),
            ("GetGamepadState".to_string(), "Array<InputObject> GetGamepadState(UserInputType gamepad)".to_string()),
            ("GetConnectedGamepads".to_string(), "Array<UserInputType> GetConnectedGamepads()".to_string()),
            ("SetNavigationGamepad".to_string(), "void SetNavigationGamepad(UserInputType gamepad, bool enabled)".to_string()),
        ]);

        map
    }

    fn find_method(&self, class_name: &str, method_name: &str, signature: &str, start: Address, end: Address) -> Option<MethodResult> {
        if let Some(addr) = self.find_method_by_string(method_name, start, end) {
            return Some(MethodResult::new(
                class_name.to_string(),
                method_name.to_string(),
                addr,
            ).with_signature(signature)
             .with_confidence(0.85));
        }

        None
    }

    fn find_method_by_string(&self, method_name: &str, start: Address, end: Address) -> Option<Address> {
        if let Some(string_addr) = self.find_string(method_name, start, end) {
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
