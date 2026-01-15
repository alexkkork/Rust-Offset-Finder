// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::{ConstantResult, ConstantValue};
use std::sync::Arc;
use std::collections::HashMap;

pub struct ConstantFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ConstantFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<ConstantResult> {
        let mut results = Vec::new();

        let known_constants = self.get_known_constants();

        for (name, expected_value) in known_constants {
            if let Some(result) = self.find_constant(&name, &expected_value, start, end) {
                results.push(result);
            }
        }

        results.extend(self.find_string_constants(start, end));

        results
    }

    fn get_known_constants(&self) -> HashMap<String, ExpectedValue> {
        let mut map = HashMap::new();

        map.insert("LUA_TNONE".to_string(), ExpectedValue::Integer(-1));
        map.insert("LUA_TNIL".to_string(), ExpectedValue::Integer(0));
        map.insert("LUA_TBOOLEAN".to_string(), ExpectedValue::Integer(1));
        map.insert("LUA_TLIGHTUSERDATA".to_string(), ExpectedValue::Integer(2));
        map.insert("LUA_TNUMBER".to_string(), ExpectedValue::Integer(3));
        map.insert("LUA_TVECTOR".to_string(), ExpectedValue::Integer(4));
        map.insert("LUA_TSTRING".to_string(), ExpectedValue::Integer(5));
        map.insert("LUA_TTABLE".to_string(), ExpectedValue::Integer(6));
        map.insert("LUA_TFUNCTION".to_string(), ExpectedValue::Integer(7));
        map.insert("LUA_TUSERDATA".to_string(), ExpectedValue::Integer(8));
        map.insert("LUA_TTHREAD".to_string(), ExpectedValue::Integer(9));
        map.insert("LUA_TBUFFER".to_string(), ExpectedValue::Integer(10));

        map.insert("LUA_OK".to_string(), ExpectedValue::Integer(0));
        map.insert("LUA_YIELD".to_string(), ExpectedValue::Integer(1));
        map.insert("LUA_ERRRUN".to_string(), ExpectedValue::Integer(2));
        map.insert("LUA_ERRSYNTAX".to_string(), ExpectedValue::Integer(3));
        map.insert("LUA_ERRMEM".to_string(), ExpectedValue::Integer(4));
        map.insert("LUA_ERRERR".to_string(), ExpectedValue::Integer(5));
        map.insert("LUA_BREAK".to_string(), ExpectedValue::Integer(6));

        map.insert("LUA_GCSTOP".to_string(), ExpectedValue::Integer(0));
        map.insert("LUA_GCRESTART".to_string(), ExpectedValue::Integer(1));
        map.insert("LUA_GCCOLLECT".to_string(), ExpectedValue::Integer(2));
        map.insert("LUA_GCCOUNT".to_string(), ExpectedValue::Integer(3));
        map.insert("LUA_GCCOUNTB".to_string(), ExpectedValue::Integer(4));
        map.insert("LUA_GCSTEP".to_string(), ExpectedValue::Integer(5));
        map.insert("LUA_GCSETPAUSE".to_string(), ExpectedValue::Integer(6));
        map.insert("LUA_GCSETSTEPMUL".to_string(), ExpectedValue::Integer(7));

        map.insert("LUA_MULTRET".to_string(), ExpectedValue::Integer(-1));
        map.insert("LUA_REGISTRYINDEX".to_string(), ExpectedValue::Integer(-10000));
        map.insert("LUA_ENVIRONINDEX".to_string(), ExpectedValue::Integer(-10001));
        map.insert("LUA_GLOBALSINDEX".to_string(), ExpectedValue::Integer(-10002));

        map.insert("LUA_MINSTACK".to_string(), ExpectedValue::Integer(20));
        map.insert("LUAI_MAXSTACK".to_string(), ExpectedValue::Integer(1000000));
        map.insert("LUAI_MAXCSTACK".to_string(), ExpectedValue::Integer(8000));

        map.insert("IDENTITY_PLUGIN".to_string(), ExpectedValue::Integer(1));
        map.insert("IDENTITY_REPLICATOR".to_string(), ExpectedValue::Integer(2));
        map.insert("IDENTITY_LOCAL_USER".to_string(), ExpectedValue::Integer(3));
        map.insert("IDENTITY_GAME_SCRIPT".to_string(), ExpectedValue::Integer(4));
        map.insert("IDENTITY_LOCAL_ROBLOX".to_string(), ExpectedValue::Integer(5));
        map.insert("IDENTITY_COM_SCRIPT".to_string(), ExpectedValue::Integer(6));
        map.insert("IDENTITY_COMMAND_BAR".to_string(), ExpectedValue::Integer(7));
        map.insert("IDENTITY_ROBLOX_SCRIPT".to_string(), ExpectedValue::Integer(8));

        map
    }

    fn find_constant(&self, name: &str, expected: &ExpectedValue, start: Address, end: Address) -> Option<ConstantResult> {
        if let Some(string_addr) = self.find_string(name, start, end) {
            let value = match expected {
                ExpectedValue::Integer(i) => ConstantValue::Integer(*i),
                ExpectedValue::Float(f) => ConstantValue::Float(*f),
                ExpectedValue::String(s) => ConstantValue::String(s.clone()),
            };

            return Some(ConstantResult::new(
                name.to_string(),
                string_addr,
                value,
            ).with_confidence(0.85));
        }

        None
    }

    fn find_string_constants(&self, start: Address, end: Address) -> Vec<ConstantResult> {
        let mut results = Vec::new();

        let interesting_strings = [
            "Roblox",
            "userdata",
            "Instance",
            "game",
            "workspace",
            "script",
            "LocalPlayer",
            "PlayerGui",
            "StarterGui",
            "CoreGui",
            "Players",
            "ReplicatedStorage",
            "ServerStorage",
            "ServerScriptService",
            "Lighting",
            "RunService",
            "UserInputService",
            "TweenService",
            "HttpService",
            "MarketplaceService",
            "DataStoreService",
            "ContextActionService",
            "GuiService",
            "SoundService",
            "TextService",
            "TeleportService",
            "PathfindingService",
            "PhysicsService",
            "CollectionService",
            "BadgeService",
            "TestService",
            "Selection",
            "PluginGuiService",
        ];

        for string in &interesting_strings {
            if let Some(addr) = self.find_string(string, start, end) {
                results.push(ConstantResult::new(
                    string.to_string(),
                    addr,
                    ConstantValue::String(string.to_string()),
                ).with_confidence(0.90));
            }
        }

        results
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

enum ExpectedValue {
    Integer(i64),
    Float(f64),
    String(String),
}
