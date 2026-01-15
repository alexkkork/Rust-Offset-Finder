// Tue Jan 15 2026 - Alex

pub mod engine;
pub mod compiler;
pub mod runtime;
pub mod api;
pub mod types;
pub mod builtins;

pub use engine::{ScriptEngine, ScriptContext, ScriptResult};
pub use compiler::{ScriptCompiler, CompiledScript, CompileError};
pub use runtime::{ScriptRuntime, RuntimeError, RuntimeValue};
pub use api::{ScriptApi, ApiFunction, ApiModule};
pub use types::{ScriptType, ScriptValue, ScriptFunction};
pub use builtins::{register_builtins, BuiltinFunctions};
