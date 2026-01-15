// Tue Jan 15 2026 - Alex

use crate::memory::MemoryReader;
use crate::scripting::types::{ScriptValue, ScriptType};
use crate::scripting::runtime::RuntimeError;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Script API for registering functions
pub struct ScriptApi {
    reader: Arc<dyn MemoryReader>,
    functions: HashMap<String, ApiFunction>,
    modules: HashMap<String, ApiModule>,
}

impl ScriptApi {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            functions: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[ScriptValue]) -> Result<ScriptValue, RuntimeError> + Send + Sync + 'static,
    {
        let api_func = ApiFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type: ScriptType::Any,
            description: String::new(),
            handler: Box::new(func),
        };
        self.functions.insert(name.to_string(), api_func);
    }

    /// Register a function with metadata
    pub fn register_function_with_meta(&mut self, func: ApiFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Register a module
    pub fn register_module(&mut self, module: ApiModule) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Call a function
    pub fn call(&self, name: &str, args: &[ScriptValue]) -> Result<ScriptValue, RuntimeError> {
        // Check direct functions first
        if let Some(func) = self.functions.get(name) {
            return (func.handler)(args);
        }

        // Check module functions (module.function format)
        if let Some(dot_idx) = name.find('.') {
            let module_name = &name[..dot_idx];
            let func_name = &name[dot_idx + 1..];
            
            if let Some(module) = self.modules.get(module_name) {
                if let Some(func) = module.functions.get(func_name) {
                    return (func.handler)(args);
                }
            }
        }

        Err(RuntimeError::NameError(format!("Function '{}' not found", name)))
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        if self.functions.contains_key(name) {
            return true;
        }

        if let Some(dot_idx) = name.find('.') {
            let module_name = &name[..dot_idx];
            let func_name = &name[dot_idx + 1..];
            
            if let Some(module) = self.modules.get(module_name) {
                return module.functions.contains_key(func_name);
            }
        }

        false
    }

    /// Get all function names
    pub fn function_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.functions.keys().cloned().collect();
        
        for (module_name, module) in &self.modules {
            for func_name in module.functions.keys() {
                names.push(format!("{}.{}", module_name, func_name));
            }
        }

        names.sort();
        names
    }

    /// Get function help
    pub fn get_help(&self, name: &str) -> Option<String> {
        if let Some(func) = self.functions.get(name) {
            return Some(func.help_text());
        }

        if let Some(dot_idx) = name.find('.') {
            let module_name = &name[..dot_idx];
            let func_name = &name[dot_idx + 1..];
            
            if let Some(module) = self.modules.get(module_name) {
                if let Some(func) = module.functions.get(func_name) {
                    return Some(func.help_text());
                }
            }
        }

        None
    }

    /// Get the memory reader
    pub fn reader(&self) -> &Arc<dyn MemoryReader> {
        &self.reader
    }
}

/// API function definition
pub struct ApiFunction {
    pub name: String,
    pub params: Vec<(String, ScriptType, bool)>, // (name, type, optional)
    pub return_type: ScriptType,
    pub description: String,
    pub handler: Box<dyn Fn(&[ScriptValue]) -> Result<ScriptValue, RuntimeError> + Send + Sync>,
}

impl ApiFunction {
    pub fn new<F>(name: &str, handler: F) -> Self
    where
        F: Fn(&[ScriptValue]) -> Result<ScriptValue, RuntimeError> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            params: Vec::new(),
            return_type: ScriptType::Any,
            description: String::new(),
            handler: Box::new(handler),
        }
    }

    pub fn with_param(mut self, name: &str, typ: ScriptType, optional: bool) -> Self {
        self.params.push((name.to_string(), typ, optional));
        self
    }

    pub fn with_return_type(mut self, typ: ScriptType) -> Self {
        self.return_type = typ;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn help_text(&self) -> String {
        let mut help = String::new();

        // Function signature
        let params: Vec<String> = self.params.iter()
            .map(|(name, typ, opt)| {
                if *opt {
                    format!("{}?: {}", name, typ)
                } else {
                    format!("{}: {}", name, typ)
                }
            })
            .collect();

        help.push_str(&format!("{}({}) -> {}\n", 
            self.name, 
            params.join(", "),
            self.return_type
        ));

        if !self.description.is_empty() {
            help.push_str(&format!("\n{}\n", self.description));
        }

        help
    }

    pub fn required_args(&self) -> usize {
        self.params.iter().filter(|(_, _, opt)| !opt).count()
    }

    pub fn max_args(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Debug for ApiFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiFunction")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("return_type", &self.return_type)
            .finish()
    }
}

/// API module containing related functions
pub struct ApiModule {
    pub name: String,
    pub description: String,
    pub functions: HashMap<String, ApiFunction>,
}

impl ApiModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            functions: HashMap::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn add_function(mut self, func: ApiFunction) -> Self {
        self.functions.insert(func.name.clone(), func);
        self
    }

    pub fn function_names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }

    pub fn help_text(&self) -> String {
        let mut help = format!("Module: {}\n", self.name);
        
        if !self.description.is_empty() {
            help.push_str(&format!("{}\n", self.description));
        }

        help.push_str("\nFunctions:\n");
        for name in self.function_names() {
            help.push_str(&format!("  {}.{}\n", self.name, name));
        }

        help
    }
}

impl fmt::Debug for ApiModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiModule")
            .field("name", &self.name)
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Builder for creating API modules
pub struct ApiModuleBuilder {
    module: ApiModule,
}

impl ApiModuleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            module: ApiModule::new(name),
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.module.description = desc.to_string();
        self
    }

    pub fn function<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(&[ScriptValue]) -> Result<ScriptValue, RuntimeError> + Send + Sync + 'static,
    {
        let func = ApiFunction::new(name, handler);
        self.module.functions.insert(name.to_string(), func);
        self
    }

    pub fn function_with_meta(mut self, func: ApiFunction) -> Self {
        self.module.functions.insert(func.name.clone(), func);
        self
    }

    pub fn build(self) -> ApiModule {
        self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_function_help() {
        let func = ApiFunction::new("test", |_| Ok(ScriptValue::Nil))
            .with_param("x", ScriptType::Integer, false)
            .with_param("y", ScriptType::Integer, true)
            .with_return_type(ScriptType::Integer)
            .with_description("Test function");

        let help = func.help_text();
        assert!(help.contains("test"));
        assert!(help.contains("Test function"));
    }

    #[test]
    fn test_api_module_builder() {
        let module = ApiModuleBuilder::new("math")
            .description("Math functions")
            .function("abs", |args| {
                if let Some(n) = args.get(0).and_then(|v| v.as_int()) {
                    Ok(ScriptValue::Integer(n.abs()))
                } else {
                    Ok(ScriptValue::Nil)
                }
            })
            .build();

        assert_eq!(module.name, "math");
        assert!(module.functions.contains_key("abs"));
    }
}
