// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::ClassResult;
use std::sync::Arc;
use std::collections::HashMap;

pub struct ReflectionFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ReflectionFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<ClassResult> {
        let mut results = Vec::new();

        if let Some(class_descriptors) = self.find_class_descriptors(start, end) {
            for (name, addr) in class_descriptors {
                results.push(ClassResult::new(name, addr)
                    .with_confidence(0.85));
            }
        }

        results
    }

    fn find_class_descriptors(&self, start: Address, end: Address) -> Option<HashMap<String, Address>> {
        let mut descriptors = HashMap::new();

        let class_names = [
            "Instance", "Part", "BasePart", "Model", "Workspace",
            "DataModel", "Player", "Players", "Camera", "Lighting",
            "ReplicatedStorage", "ServerStorage", "ServerScriptService",
            "StarterGui", "StarterPack", "StarterPlayer", "Teams",
            "SoundService", "Chat", "LocalizationService", "TestService",
            "RunService", "UserInputService", "ContextActionService",
            "GuiService", "HapticService", "TweenService", "TeleportService",
            "TextService", "ContentProvider", "Debris", "PathfindingService",
            "BadgeService", "NotificationService", "MarketplaceService",
            "AssetService", "InsertService", "PointsService", "GamePassService",
            "GroupService", "HttpService", "KeyframeSequenceProvider",
            "LogService", "ScriptContext", "Selection", "Stats",
            "StudioService", "PluginGuiService", "VirtualInputManager",
            "Script", "LocalScript", "ModuleScript", "CoreScript",
            "RemoteEvent", "RemoteFunction", "BindableEvent", "BindableFunction",
            "Folder", "Configuration", "StringValue", "IntValue", "NumberValue",
            "BoolValue", "ObjectValue", "CFrameValue", "Vector3Value", "Color3Value",
            "BrickColorValue", "RayValue", "Part", "WedgePart", "CornerWedgePart",
            "SpawnLocation", "Seat", "VehicleSeat", "SkateboardPlatform",
            "Truss", "MeshPart", "UnionOperation", "NegateOperation",
            "Terrain", "Humanoid", "HumanoidDescription", "Animator",
            "Animation", "AnimationTrack", "KeyframeSequence", "Keyframe",
            "Pose", "Motor6D", "Weld", "Snap", "Rotate", "Glue", "ManualWeld",
            "ManualGlue", "VelocityMotor", "Attachment", "Constraint",
            "BallSocketConstraint", "HingeConstraint", "PrismaticConstraint",
            "CylindricalConstraint", "RodConstraint", "RopeConstraint",
            "SpringConstraint", "AlignOrientation", "AlignPosition",
            "AngularVelocity", "LinearVelocity", "VectorForce", "Torque",
            "BodyForce", "BodyVelocity", "BodyAngularVelocity", "BodyPosition",
            "BodyGyro", "BodyThrust", "RocketPropulsion", "BodyMover",
            "Sound", "SoundGroup", "SoundEffect", "ChorusSoundEffect",
            "CompressorSoundEffect", "DistortionSoundEffect", "EchoSoundEffect",
            "EqualizerSoundEffect", "FlangeSoundEffect", "PitchShiftSoundEffect",
            "ReverbSoundEffect", "TremoloSoundEffect", "ParticleEmitter",
            "Fire", "Smoke", "Sparkles", "Explosion", "ForceField",
            "Decal", "Texture", "SurfaceGui", "BillboardGui", "ScreenGui",
            "AdGui", "SurfaceSelection", "Handles", "ArcHandles",
            "Frame", "TextLabel", "TextButton", "TextBox", "ImageLabel",
            "ImageButton", "ViewportFrame", "ScrollingFrame", "UIListLayout",
            "UIGridLayout", "UITableLayout", "UIPageLayout", "UIPadding",
            "UIScale", "UIAspectRatioConstraint", "UISizeConstraint",
            "UITextSizeConstraint", "UICorner", "UIStroke", "UIGradient",
            "Beam", "Trail", "Highlight", "Tool", "HopperBin",
            "BackpackItem", "Backpack", "ProximityPrompt", "ClickDetector",
            "Dialog", "DialogChoice", "CharacterMesh", "Accessory", "Accoutrement",
            "Hat", "Shirt", "Pants", "ShirtGraphic", "CharacterAppearance",
            "Skin", "BodyColors", "Clothing", "ClothingItem", "Face",
        ];

        for class_name in &class_names {
            if let Some(addr) = self.find_class_descriptor(class_name, start, end) {
                descriptors.insert(class_name.to_string(), addr);
            }
        }

        if descriptors.is_empty() {
            None
        } else {
            Some(descriptors)
        }
    }

    fn find_class_descriptor(&self, class_name: &str, start: Address, end: Address) -> Option<Address> {
        if let Some(string_addr) = self.find_string(class_name, start, end) {
            let search_start = if start.as_u64() > 0x1000 {
                start - 0x1000
            } else {
                start
            };

            let mut current = search_start;
            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    for i in (0..bytes.len() - 8).step_by(8) {
                        let ptr = u64::from_le_bytes([
                            bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3],
                            bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7],
                        ]);

                        if ptr == string_addr.as_u64() {
                            let potential_descriptor = current + i as u64 - 8;

                            if self.validate_class_descriptor(potential_descriptor) {
                                return Some(potential_descriptor);
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        None
    }

    fn validate_class_descriptor(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 64) {
            let vtable_ptr = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);

            if vtable_ptr < 0x100000000 || vtable_ptr > 0x7FFFFFFFFFFF {
                return false;
            }

            let name_ptr = u64::from_le_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11],
                bytes[12], bytes[13], bytes[14], bytes[15],
            ]);

            if name_ptr < 0x100000000 || name_ptr > 0x7FFFFFFFFFFF {
                return false;
            }

            return true;
        }

        false
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
}
