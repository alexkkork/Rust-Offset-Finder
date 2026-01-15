// Tue Jan 15 2026 - Alex

use crate::memory::MemoryReader;
use crate::scripting::runtime::{ScriptRuntime, RuntimeError};
use crate::scripting::compiler::{ScriptCompiler, CompiledScript, CompileError};
use crate::scripting::api::ScriptApi;
use crate::scripting::types::ScriptValue;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Script execution engine
pub struct ScriptEngine {
    reader: Arc<dyn MemoryReader>,
    runtime: ScriptRuntime,
    compiler: ScriptCompiler,
    api: ScriptApi,
    scripts: HashMap<String, CompiledScript>,
    global_vars: HashMap<String, ScriptValue>,
    execution_limit: usize,
    memory_limit: usize,
}

impl ScriptEngine {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let mut engine = Self {
            reader: reader.clone(),
            runtime: ScriptRuntime::new(reader.clone()),
            compiler: ScriptCompiler::new(),
            api: ScriptApi::new(reader),
            scripts: HashMap::new(),
            global_vars: HashMap::new(),
            execution_limit: 1_000_000,
            memory_limit: 64 * 1024 * 1024,
        };

        // Register built-in functions
        crate::scripting::builtins::register_builtins(&mut engine.api);

        engine
    }

    pub fn with_execution_limit(mut self, limit: usize) -> Self {
        self.execution_limit = limit;
        self
    }

    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Load and compile a script
    pub fn load_script(&mut self, name: &str, source: &str) -> Result<(), ScriptError> {
        let compiled = self.compiler.compile(source)
            .map_err(|e| ScriptError::CompileError(e))?;
        
        self.scripts.insert(name.to_string(), compiled);
        Ok(())
    }

    /// Execute a loaded script
    pub fn execute(&mut self, name: &str) -> Result<ScriptResult, ScriptError> {
        let script = self.scripts.get(name)
            .ok_or_else(|| ScriptError::ScriptNotFound(name.to_string()))?
            .clone();

        let mut ctx = ScriptContext::new(&self.api, &self.global_vars);
        ctx.execution_limit = self.execution_limit;
        ctx.memory_limit = self.memory_limit;

        self.runtime.execute(&script, &mut ctx)
            .map_err(|e| ScriptError::RuntimeError(e))
    }

    /// Execute script source directly
    pub fn eval(&mut self, source: &str) -> Result<ScriptResult, ScriptError> {
        let compiled = self.compiler.compile(source)
            .map_err(|e| ScriptError::CompileError(e))?;

        let mut ctx = ScriptContext::new(&self.api, &self.global_vars);
        ctx.execution_limit = self.execution_limit;

        self.runtime.execute(&compiled, &mut ctx)
            .map_err(|e| ScriptError::RuntimeError(e))
    }

    /// Set a global variable
    pub fn set_global(&mut self, name: &str, value: ScriptValue) {
        self.global_vars.insert(name.to_string(), value);
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<&ScriptValue> {
        self.global_vars.get(name)
    }

    /// Register a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[ScriptValue]) -> Result<ScriptValue, RuntimeError> + Send + Sync + 'static,
    {
        self.api.register_function(name, func);
    }

    /// Get list of available functions
    pub fn available_functions(&self) -> Vec<String> {
        self.api.function_names()
    }

    /// Get list of loaded scripts
    pub fn loaded_scripts(&self) -> Vec<&str> {
        self.scripts.keys().map(|s| s.as_str()).collect()
    }

    /// Unload a script
    pub fn unload_script(&mut self, name: &str) -> bool {
        self.scripts.remove(name).is_some()
    }

    /// Clear all loaded scripts
    pub fn clear_scripts(&mut self) {
        self.scripts.clear();
    }

    /// Get the memory reader
    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

/// Execution context for scripts
pub struct ScriptContext<'a> {
    api: &'a ScriptApi,
    globals: &'a HashMap<String, ScriptValue>,
    locals: HashMap<String, ScriptValue>,
    call_stack: Vec<CallFrame>,
    execution_count: usize,
    pub execution_limit: usize,
    memory_used: usize,
    pub memory_limit: usize,
    return_value: Option<ScriptValue>,
}

impl<'a> ScriptContext<'a> {
    pub fn new(api: &'a ScriptApi, globals: &'a HashMap<String, ScriptValue>) -> Self {
        Self {
            api,
            globals,
            locals: HashMap::new(),
            call_stack: Vec::new(),
            execution_count: 0,
            execution_limit: 1_000_000,
            memory_used: 0,
            memory_limit: 64 * 1024 * 1024,
            return_value: None,
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&ScriptValue> {
        self.locals.get(name).or_else(|| self.globals.get(name))
    }

    pub fn set_variable(&mut self, name: &str, value: ScriptValue) {
        let size = value.memory_size();
        self.memory_used += size;
        self.locals.insert(name.to_string(), value);
    }

    pub fn call_function(&mut self, name: &str, args: &[ScriptValue]) -> Result<ScriptValue, RuntimeError> {
        self.execution_count += 1;
        if self.execution_count > self.execution_limit {
            return Err(RuntimeError::ExecutionLimitExceeded);
        }

        self.api.call(name, args)
    }

    pub fn push_frame(&mut self, name: &str) {
        self.call_stack.push(CallFrame {
            function_name: name.to_string(),
            locals: std::mem::take(&mut self.locals),
        });
    }

    pub fn pop_frame(&mut self) -> Option<CallFrame> {
        let frame = self.call_stack.pop();
        if let Some(ref f) = frame {
            self.locals = f.locals.clone();
        }
        frame
    }

    pub fn stack_depth(&self) -> usize {
        self.call_stack.len()
    }

    pub fn set_return(&mut self, value: ScriptValue) {
        self.return_value = Some(value);
    }

    pub fn take_return(&mut self) -> Option<ScriptValue> {
        self.return_value.take()
    }

    pub fn check_memory(&self) -> Result<(), RuntimeError> {
        if self.memory_used > self.memory_limit {
            Err(RuntimeError::MemoryLimitExceeded)
        } else {
            Ok(())
        }
    }
}

/// Call stack frame
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub locals: HashMap<String, ScriptValue>,
}

/// Script execution result
#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub value: ScriptValue,
    pub execution_time_ms: u64,
    pub instructions_executed: usize,
    pub memory_used: usize,
    pub output: Vec<String>,
}

impl ScriptResult {
    pub fn new(value: ScriptValue) -> Self {
        Self {
            value,
            execution_time_ms: 0,
            instructions_executed: 0,
            memory_used: 0,
            output: Vec::new(),
        }
    }

    pub fn with_stats(mut self, time_ms: u64, instructions: usize, memory: usize) -> Self {
        self.execution_time_ms = time_ms;
        self.instructions_executed = instructions;
        self.memory_used = memory;
        self
    }

    pub fn is_nil(&self) -> bool {
        matches!(self.value, ScriptValue::Nil)
    }

    pub fn as_int(&self) -> Option<i64> {
        match &self.value {
            ScriptValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match &self.value {
            ScriptValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.value {
            ScriptValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for ScriptResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        if !self.output.is_empty() {
            writeln!(f)?;
            for line in &self.output {
                writeln!(f, "{}", line)?;
            }
        }
        Ok(())
    }
}

/// Script errors
#[derive(Debug, Clone)]
pub enum ScriptError {
    CompileError(CompileError),
    RuntimeError(RuntimeError),
    ScriptNotFound(String),
    InvalidArgument(String),
    IoError(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::CompileError(e) => write!(f, "Compile error: {}", e),
            ScriptError::RuntimeError(e) => write!(f, "Runtime error: {}", e),
            ScriptError::ScriptNotFound(name) => write!(f, "Script not found: {}", name),
            ScriptError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            ScriptError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Script loader for loading scripts from files
pub struct ScriptLoader {
    search_paths: Vec<String>,
    loaded: HashMap<String, String>,
}

impl ScriptLoader {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            loaded: HashMap::new(),
        }
    }

    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(path.to_string());
    }

    pub fn load(&mut self, name: &str) -> Result<&str, ScriptError> {
        if self.loaded.contains_key(name) {
            return Ok(self.loaded.get(name).unwrap());
        }

        // Try to find the script file
        for search_path in &self.search_paths {
            let full_path = format!("{}/{}.script", search_path, name);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                self.loaded.insert(name.to_string(), content);
                return Ok(self.loaded.get(name).unwrap());
            }
        }

        Err(ScriptError::ScriptNotFound(name.to_string()))
    }

    pub fn reload(&mut self, name: &str) -> Result<&str, ScriptError> {
        self.loaded.remove(name);
        self.load(name)
    }
}

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_result() {
        let result = ScriptResult::new(ScriptValue::Integer(42));
        assert_eq!(result.as_int(), Some(42));
        assert!(!result.is_nil());
    }

    #[test]
    fn test_script_error_display() {
        let err = ScriptError::ScriptNotFound("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
