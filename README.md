# Roblox Offset Finder (In Progress)

A high-performance ARM64 offset generator for macOS Roblox, written in Rust.

## Status

🚧 **Work in Progress** - This tool is actively being developed and is not yet ready for production use.

## Features

- **Pattern Scanning** - Fast byte pattern matching with wildcard support
- **Symbol Resolution** - Mach-O symbol parsing and C++ demangling
- **Cross-Reference Analysis** - Build and analyze call graphs
- **Structure Recovery** - Infer Lua and Roblox internal structures
- **Heuristic Detection** - Smart offset discovery using multiple strategies

## Supported Offsets

### Lua API Functions
- `lua_gettop`, `lua_settop`, `lua_pushvalue`, `lua_type`
- `lua_tonumber`, `lua_tostring`, `lua_toboolean`
- `lua_newthread`, `lua_newuserdata`, `lua_newstate`
- And more...

### Roblox Functions
- `LuauLoad`, `PushInstance`, `GetTypename`
- `taskSpawn`, `taskDefer`, `sctxResume`
- `CreateJob`, `TaskScheduler`
- And more...

### Structure Offsets
- `lua_State` internals (top, stack, ci, global)
- `ExtraSpace` / ScriptContext
- `Closure`, `Proto`, `Table`, `TValue`
- Class descriptors and vtables

## Building

```bash
cargo build --release
```

## Usage

```bash
# Generate offsets from a binary
./roblox-offset-generator generate --binary /path/to/RobloxPlayer

# Generate offsets from a running process
./roblox-offset-generator generate --process "RobloxPlayer"

# Compare two offset files
./roblox-offset-generator diff --old offsets_v1.json --new offsets_v2.json
```

## Output Format

Offsets are exported to `offsets.json`:

```json
{
  "version": "1.0.0",
  "timestamp": "2026-01-15T00:00:00Z",
  "functions": {
    "lua_gettop": {
      "address": "0x1234567890",
      "confidence": 0.95
    }
  },
  "structure_offsets": {
    "lua_State": {
      "top": 16,
      "stack": 24,
      "ci": 40
    }
  }
}
```

## Requirements

- macOS (ARM64)
- Rust 1.70+

## License

MIT License - see [LICENSE](LICENSE)

## Author

Alex - 2026
