// Tue Jan 13 2026 - Alex

use std::collections::HashMap;

pub struct LuaTypeConstants {
    pub none: i32,
    pub nil: i32,
    pub boolean: i32,
    pub lightuserdata: i32,
    pub number: i32,
    pub vector: i32,
    pub string: i32,
    pub table: i32,
    pub function: i32,
    pub userdata: i32,
    pub thread: i32,
    pub buffer: i32,
}

impl Default for LuaTypeConstants {
    fn default() -> Self {
        Self {
            none: -1,
            nil: 0,
            boolean: 1,
            lightuserdata: 2,
            number: 3,
            vector: 4,
            string: 5,
            table: 6,
            function: 7,
            userdata: 8,
            thread: 9,
            buffer: 10,
        }
    }
}

pub struct LuaStatusConstants {
    pub ok: i32,
    pub yield_status: i32,
    pub errrun: i32,
    pub errsyntax: i32,
    pub errmem: i32,
    pub errerr: i32,
    pub break_status: i32,
}

impl Default for LuaStatusConstants {
    fn default() -> Self {
        Self {
            ok: 0,
            yield_status: 1,
            errrun: 2,
            errsyntax: 3,
            errmem: 4,
            errerr: 5,
            break_status: 6,
        }
    }
}

pub struct IdentityConstants {
    pub plugin: i32,
    pub replicator: i32,
    pub local_user: i32,
    pub game_script: i32,
    pub local_roblox: i32,
    pub com_script: i32,
    pub command_bar: i32,
    pub roblox_script: i32,
}

impl Default for IdentityConstants {
    fn default() -> Self {
        Self {
            plugin: 1,
            replicator: 2,
            local_user: 3,
            game_script: 4,
            local_roblox: 5,
            com_script: 6,
            command_bar: 7,
            roblox_script: 8,
        }
    }
}

pub struct SpecialIndexConstants {
    pub multret: i32,
    pub registry_index: i32,
    pub environ_index: i32,
    pub globals_index: i32,
}

impl Default for SpecialIndexConstants {
    fn default() -> Self {
        Self {
            multret: -1,
            registry_index: -10000,
            environ_index: -10001,
            globals_index: -10002,
        }
    }
}

pub fn get_type_name(type_tag: i32) -> &'static str {
    match type_tag {
        -1 => "none",
        0 => "nil",
        1 => "boolean",
        2 => "lightuserdata",
        3 => "number",
        4 => "vector",
        5 => "string",
        6 => "table",
        7 => "function",
        8 => "userdata",
        9 => "thread",
        10 => "buffer",
        _ => "unknown",
    }
}

pub fn get_type_tag(type_name: &str) -> Option<i32> {
    match type_name.to_lowercase().as_str() {
        "none" => Some(-1),
        "nil" => Some(0),
        "boolean" | "bool" => Some(1),
        "lightuserdata" => Some(2),
        "number" => Some(3),
        "vector" => Some(4),
        "string" => Some(5),
        "table" => Some(6),
        "function" => Some(7),
        "userdata" => Some(8),
        "thread" => Some(9),
        "buffer" => Some(10),
        _ => None,
    }
}

pub fn get_status_name(status: i32) -> &'static str {
    match status {
        0 => "OK",
        1 => "YIELD",
        2 => "ERRRUN",
        3 => "ERRSYNTAX",
        4 => "ERRMEM",
        5 => "ERRERR",
        6 => "BREAK",
        _ => "UNKNOWN",
    }
}

pub fn get_identity_name(identity: i32) -> &'static str {
    match identity {
        1 => "Plugin",
        2 => "Replicator",
        3 => "LocalUser",
        4 => "GameScript",
        5 => "LocalRoblox",
        6 => "ComScript",
        7 => "CommandBar",
        8 => "RobloxScript",
        _ => "Unknown",
    }
}

pub fn build_constant_map() -> HashMap<String, i64> {
    let mut map = HashMap::new();

    map.insert("LUA_TNONE".to_string(), -1);
    map.insert("LUA_TNIL".to_string(), 0);
    map.insert("LUA_TBOOLEAN".to_string(), 1);
    map.insert("LUA_TLIGHTUSERDATA".to_string(), 2);
    map.insert("LUA_TNUMBER".to_string(), 3);
    map.insert("LUA_TVECTOR".to_string(), 4);
    map.insert("LUA_TSTRING".to_string(), 5);
    map.insert("LUA_TTABLE".to_string(), 6);
    map.insert("LUA_TFUNCTION".to_string(), 7);
    map.insert("LUA_TUSERDATA".to_string(), 8);
    map.insert("LUA_TTHREAD".to_string(), 9);
    map.insert("LUA_TBUFFER".to_string(), 10);

    map.insert("LUA_OK".to_string(), 0);
    map.insert("LUA_YIELD".to_string(), 1);
    map.insert("LUA_ERRRUN".to_string(), 2);
    map.insert("LUA_ERRSYNTAX".to_string(), 3);
    map.insert("LUA_ERRMEM".to_string(), 4);
    map.insert("LUA_ERRERR".to_string(), 5);
    map.insert("LUA_BREAK".to_string(), 6);

    map.insert("LUA_MULTRET".to_string(), -1);
    map.insert("LUA_REGISTRYINDEX".to_string(), -10000);
    map.insert("LUA_ENVIRONINDEX".to_string(), -10001);
    map.insert("LUA_GLOBALSINDEX".to_string(), -10002);

    map.insert("IDENTITY_PLUGIN".to_string(), 1);
    map.insert("IDENTITY_REPLICATOR".to_string(), 2);
    map.insert("IDENTITY_LOCAL_USER".to_string(), 3);
    map.insert("IDENTITY_GAME_SCRIPT".to_string(), 4);
    map.insert("IDENTITY_LOCAL_ROBLOX".to_string(), 5);
    map.insert("IDENTITY_COM_SCRIPT".to_string(), 6);
    map.insert("IDENTITY_COMMAND_BAR".to_string(), 7);
    map.insert("IDENTITY_ROBLOX_SCRIPT".to_string(), 8);

    map
}
