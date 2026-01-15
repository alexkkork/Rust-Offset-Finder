// Tue Jan 15 2026 - Alex

pub mod bytecode;
pub mod opcode;
pub mod compiler;
pub mod vm;
pub mod state;
pub mod types;
pub mod gc;
pub mod debug;
pub mod api;
pub mod decompiler;
pub mod upvalue;

pub use bytecode::LuauBytecode;
pub use opcode::LuauOpcode;
pub use vm::VmAnalyzer;
pub use state::StateAnalyzer;
pub use types::{LuauType, TypeTag, TValue};
pub use gc::GcAnalyzer;
pub use debug::DebugInfoAnalyzer;
pub use api::LuauApi;
pub use decompiler::{LuauDecompiler, DecompilationResult, BytecodeAnalyzer, BytecodeAnalysis, Constant, ConstantPropagation};
pub use upvalue::{Upvalue, UpvalueState, UpvalueAnalyzer, UpvalueRefMap};
