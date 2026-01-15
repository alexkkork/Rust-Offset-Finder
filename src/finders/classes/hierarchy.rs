// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: String,
    pub address: Address,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub vtable: Option<Address>,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct ClassHierarchy {
    nodes: HashMap<String, ClassNode>,
    root_classes: Vec<String>,
}

impl ClassHierarchy {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_classes: Vec::new(),
        }
    }

    pub fn add_class(&mut self, name: String, address: Address, parent: Option<String>, vtable: Option<Address>) {
        let depth = if let Some(ref parent_name) = parent {
            self.nodes.get(parent_name)
                .map(|p| p.depth + 1)
                .unwrap_or(0)
        } else {
            0
        };

        let node = ClassNode {
            name: name.clone(),
            address,
            parent: parent.clone(),
            children: Vec::new(),
            vtable,
            depth,
        };

        if let Some(ref parent_name) = parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_name) {
                parent_node.children.push(name.clone());
            }
        } else {
            self.root_classes.push(name.clone());
        }

        self.nodes.insert(name, node);
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassNode> {
        self.nodes.get(name)
    }

    pub fn get_parent(&self, name: &str) -> Option<&ClassNode> {
        self.nodes.get(name)
            .and_then(|node| node.parent.as_ref())
            .and_then(|parent_name| self.nodes.get(parent_name))
    }

    pub fn get_children(&self, name: &str) -> Vec<&ClassNode> {
        self.nodes.get(name)
            .map(|node| {
                node.children.iter()
                    .filter_map(|child_name| self.nodes.get(child_name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_ancestors(&self, name: &str) -> Vec<&ClassNode> {
        let mut ancestors = Vec::new();
        let mut current = self.get_class(name);

        while let Some(node) = current {
            if let Some(parent) = self.get_parent(&node.name) {
                ancestors.push(parent);
                current = Some(parent);
            } else {
                break;
            }
        }

        ancestors
    }

    pub fn get_descendants(&self, name: &str) -> Vec<&ClassNode> {
        let mut descendants = Vec::new();
        let mut queue: Vec<&str> = vec![name];

        while let Some(current_name) = queue.pop() {
            for child in self.get_children(current_name) {
                descendants.push(child);
                queue.push(&child.name);
            }
        }

        descendants
    }

    pub fn is_descendant_of(&self, name: &str, ancestor: &str) -> bool {
        let ancestors = self.get_ancestors(name);
        ancestors.iter().any(|node| node.name == ancestor)
    }

    pub fn get_root_classes(&self) -> Vec<&ClassNode> {
        self.root_classes.iter()
            .filter_map(|name| self.nodes.get(name))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ClassNode> {
        self.nodes.values()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn max_depth(&self) -> usize {
        self.nodes.values()
            .map(|node| node.depth)
            .max()
            .unwrap_or(0)
    }

    pub fn to_tree_string(&self) -> String {
        let mut result = String::new();

        for root in self.get_root_classes() {
            self.build_tree_string(&mut result, root, 0);
        }

        result
    }

    fn build_tree_string(&self, result: &mut String, node: &ClassNode, indent: usize) {
        let prefix = "  ".repeat(indent);
        result.push_str(&format!("{}{}\n", prefix, node.name));

        for child in self.get_children(&node.name) {
            self.build_tree_string(result, child, indent + 1);
        }
    }
}

impl Default for ClassHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_roblox_hierarchy() -> ClassHierarchy {
    let mut hierarchy = ClassHierarchy::new();

    hierarchy.add_class("Instance".to_string(), Address::new(0), None, None);

    hierarchy.add_class("ServiceProvider".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("DataModel".to_string(), Address::new(0), Some("ServiceProvider".to_string()), None);

    hierarchy.add_class("PVInstance".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("BasePart".to_string(), Address::new(0), Some("PVInstance".to_string()), None);
    hierarchy.add_class("Part".to_string(), Address::new(0), Some("BasePart".to_string()), None);
    hierarchy.add_class("WedgePart".to_string(), Address::new(0), Some("BasePart".to_string()), None);
    hierarchy.add_class("CornerWedgePart".to_string(), Address::new(0), Some("BasePart".to_string()), None);
    hierarchy.add_class("MeshPart".to_string(), Address::new(0), Some("BasePart".to_string()), None);
    hierarchy.add_class("SpawnLocation".to_string(), Address::new(0), Some("Part".to_string()), None);
    hierarchy.add_class("Seat".to_string(), Address::new(0), Some("Part".to_string()), None);
    hierarchy.add_class("VehicleSeat".to_string(), Address::new(0), Some("BasePart".to_string()), None);
    hierarchy.add_class("Terrain".to_string(), Address::new(0), Some("BasePart".to_string()), None);

    hierarchy.add_class("Model".to_string(), Address::new(0), Some("PVInstance".to_string()), None);
    hierarchy.add_class("Workspace".to_string(), Address::new(0), Some("Model".to_string()), None);
    hierarchy.add_class("WorldRoot".to_string(), Address::new(0), Some("Model".to_string()), None);

    hierarchy.add_class("LuaSourceContainer".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("BaseScript".to_string(), Address::new(0), Some("LuaSourceContainer".to_string()), None);
    hierarchy.add_class("Script".to_string(), Address::new(0), Some("BaseScript".to_string()), None);
    hierarchy.add_class("LocalScript".to_string(), Address::new(0), Some("BaseScript".to_string()), None);
    hierarchy.add_class("ModuleScript".to_string(), Address::new(0), Some("LuaSourceContainer".to_string()), None);
    hierarchy.add_class("CoreScript".to_string(), Address::new(0), Some("BaseScript".to_string()), None);

    hierarchy.add_class("ValueBase".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("IntValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("NumberValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("StringValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("BoolValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("ObjectValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("CFrameValue".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("Vector3Value".to_string(), Address::new(0), Some("ValueBase".to_string()), None);
    hierarchy.add_class("Color3Value".to_string(), Address::new(0), Some("ValueBase".to_string()), None);

    hierarchy.add_class("GuiObject".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("Frame".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("TextLabel".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("TextButton".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("TextBox".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("ImageLabel".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("ImageButton".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("ScrollingFrame".to_string(), Address::new(0), Some("GuiObject".to_string()), None);
    hierarchy.add_class("ViewportFrame".to_string(), Address::new(0), Some("GuiObject".to_string()), None);

    hierarchy.add_class("BasePlayerGui".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("PlayerGui".to_string(), Address::new(0), Some("BasePlayerGui".to_string()), None);
    hierarchy.add_class("StarterGui".to_string(), Address::new(0), Some("BasePlayerGui".to_string()), None);

    hierarchy.add_class("LayerCollector".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("ScreenGui".to_string(), Address::new(0), Some("LayerCollector".to_string()), None);
    hierarchy.add_class("SurfaceGui".to_string(), Address::new(0), Some("LayerCollector".to_string()), None);
    hierarchy.add_class("BillboardGui".to_string(), Address::new(0), Some("LayerCollector".to_string()), None);

    hierarchy.add_class("Humanoid".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("Player".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("Players".to_string(), Address::new(0), Some("Instance".to_string()), None);
    hierarchy.add_class("Camera".to_string(), Address::new(0), Some("Instance".to_string()), None);

    hierarchy
}
