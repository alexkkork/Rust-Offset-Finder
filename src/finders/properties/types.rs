// Tue Jan 13 2026 - Alex

use std::collections::HashMap;

pub struct PropertyTypeInfo {
    pub name: &'static str,
    pub size: usize,
    pub alignment: usize,
    pub is_value_type: bool,
}

pub fn get_property_type_info() -> HashMap<&'static str, PropertyTypeInfo> {
    let mut map = HashMap::new();

    map.insert("bool", PropertyTypeInfo {
        name: "bool",
        size: 1,
        alignment: 1,
        is_value_type: true,
    });

    map.insert("int", PropertyTypeInfo {
        name: "int",
        size: 4,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("int64", PropertyTypeInfo {
        name: "int64",
        size: 8,
        alignment: 8,
        is_value_type: true,
    });

    map.insert("float", PropertyTypeInfo {
        name: "float",
        size: 4,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("double", PropertyTypeInfo {
        name: "double",
        size: 8,
        alignment: 8,
        is_value_type: true,
    });

    map.insert("string", PropertyTypeInfo {
        name: "string",
        size: 8,
        alignment: 8,
        is_value_type: false,
    });

    map.insert("Vector2", PropertyTypeInfo {
        name: "Vector2",
        size: 8,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Vector3", PropertyTypeInfo {
        name: "Vector3",
        size: 12,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("CFrame", PropertyTypeInfo {
        name: "CFrame",
        size: 48,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Color3", PropertyTypeInfo {
        name: "Color3",
        size: 12,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("BrickColor", PropertyTypeInfo {
        name: "BrickColor",
        size: 4,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("UDim", PropertyTypeInfo {
        name: "UDim",
        size: 8,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("UDim2", PropertyTypeInfo {
        name: "UDim2",
        size: 16,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Ray", PropertyTypeInfo {
        name: "Ray",
        size: 24,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Rect", PropertyTypeInfo {
        name: "Rect",
        size: 16,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Region3", PropertyTypeInfo {
        name: "Region3",
        size: 24,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("NumberRange", PropertyTypeInfo {
        name: "NumberRange",
        size: 8,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("NumberSequence", PropertyTypeInfo {
        name: "NumberSequence",
        size: 8,
        alignment: 8,
        is_value_type: false,
    });

    map.insert("ColorSequence", PropertyTypeInfo {
        name: "ColorSequence",
        size: 8,
        alignment: 8,
        is_value_type: false,
    });

    map.insert("Instance", PropertyTypeInfo {
        name: "Instance",
        size: 8,
        alignment: 8,
        is_value_type: false,
    });

    map.insert("Enum", PropertyTypeInfo {
        name: "Enum",
        size: 4,
        alignment: 4,
        is_value_type: true,
    });

    map.insert("Content", PropertyTypeInfo {
        name: "Content",
        size: 8,
        alignment: 8,
        is_value_type: false,
    });

    map
}

pub fn get_type_size(type_name: &str) -> Option<usize> {
    get_property_type_info().get(type_name).map(|info| info.size)
}

pub fn get_type_alignment(type_name: &str) -> Option<usize> {
    get_property_type_info().get(type_name).map(|info| info.alignment)
}

pub fn is_value_type(type_name: &str) -> bool {
    get_property_type_info()
        .get(type_name)
        .map(|info| info.is_value_type)
        .unwrap_or(false)
}
