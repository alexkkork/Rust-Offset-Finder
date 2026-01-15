// Tue Jan 13 2026 - Alex

pub mod lua_state;
pub mod extraspace;
pub mod closure;
pub mod proto;
pub mod table;
pub mod userdata;
pub mod string_obj;
pub mod tvalue;
pub mod gc_object;

pub use lua_state::LuaStateFinder;
pub use extraspace::ExtraspaceFinder;
pub use closure::ClosureFinder;
pub use proto::ProtoFinder;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub fn find_all_structures(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Vec<StructureOffsetResult> {
    let mut results = Vec::new();

    let lua_state_finder = LuaStateFinder::new(reader.clone());
    results.extend(lua_state_finder.find_all(start, end));

    let extraspace_finder = ExtraspaceFinder::new(reader.clone());
    results.extend(extraspace_finder.find_all(start, end));

    let closure_finder = ClosureFinder::new(reader.clone());
    results.extend(closure_finder.find_all(start, end));

    let proto_finder = ProtoFinder::new(reader.clone());
    results.extend(proto_finder.find_all(start, end));

    results
}
