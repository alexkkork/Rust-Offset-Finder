// Tue Jan 15 2026 - Alex

use crate::memory::MemoryReader;
use crate::scripting::engine::ScriptContext;
use crate::scripting::compiler::{CompiledScript, Instruction};
use crate::scripting::types::ScriptValue;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;

/// Script runtime for executing compiled bytecode
pub struct ScriptRuntime {
    reader: Arc<dyn MemoryReader>,
    stack: Vec<RuntimeValue>,
    globals: HashMap<String, RuntimeValue>,
}

impl ScriptRuntime {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    /// Execute a compiled script
    pub fn execute(&mut self, script: &CompiledScript, ctx: &mut ScriptContext) -> Result<crate::scripting::engine::ScriptResult, RuntimeError> {
        let start_time = std::time::Instant::now();
        let mut ip = 0;
        let mut instructions_executed = 0;

        while ip < script.bytecode.len() {
            instructions_executed += 1;
            
            if instructions_executed > ctx.execution_limit {
                return Err(RuntimeError::ExecutionLimitExceeded);
            }

            let instr = &script.bytecode[ip];
            ip += 1;

            match instr {
                Instruction::LoadConst(idx) => {
                    if let Some(value) = script.constants.get(*idx) {
                        self.stack.push(RuntimeValue::from_script_value(value.clone()));
                    }
                }
                Instruction::LoadNil => {
                    self.stack.push(RuntimeValue::Nil);
                }
                Instruction::LoadTrue => {
                    self.stack.push(RuntimeValue::Boolean(true));
                }
                Instruction::LoadFalse => {
                    self.stack.push(RuntimeValue::Boolean(false));
                }
                Instruction::GetLocal(slot) => {
                    let name = format!("__local_{}", slot);
                    if let Some(value) = ctx.get_variable(&name) {
                        self.stack.push(RuntimeValue::from_script_value(value.clone()));
                    } else {
                        self.stack.push(RuntimeValue::Nil);
                    }
                }
                Instruction::SetLocal(slot) => {
                    let value = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let name = format!("__local_{}", slot);
                    ctx.set_variable(&name, value.to_script_value());
                }
                Instruction::GetGlobal(name) => {
                    if let Some(value) = self.globals.get(name) {
                        self.stack.push(value.clone());
                    } else if let Some(value) = ctx.get_variable(name) {
                        self.stack.push(RuntimeValue::from_script_value(value.clone()));
                    } else {
                        self.stack.push(RuntimeValue::Nil);
                    }
                }
                Instruction::SetGlobal(name) => {
                    let value = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.globals.insert(name.clone(), value);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    if let Some(top) = self.stack.last().cloned() {
                        self.stack.push(top);
                    }
                }
                Instruction::Add => self.binary_op(|a, b| a + b)?,
                Instruction::Sub => self.binary_op(|a, b| a - b)?,
                Instruction::Mul => self.binary_op(|a, b| a * b)?,
                Instruction::Div => self.binary_op(|a, b| {
                    if b == 0.0 { f64::NAN } else { a / b }
                })?,
                Instruction::Mod => self.binary_op(|a, b| a % b)?,
                Instruction::Neg => {
                    let val = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    match val {
                        RuntimeValue::Integer(n) => self.stack.push(RuntimeValue::Integer(-n)),
                        RuntimeValue::Float(n) => self.stack.push(RuntimeValue::Float(-n)),
                        _ => return Err(RuntimeError::TypeError("Cannot negate non-number".to_string())),
                    }
                }
                Instruction::Eq => {
                    let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.stack.push(RuntimeValue::Boolean(a == b));
                }
                Instruction::Ne => {
                    let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.stack.push(RuntimeValue::Boolean(a != b));
                }
                Instruction::Lt => self.compare_op(|a, b| a < b)?,
                Instruction::Le => self.compare_op(|a, b| a <= b)?,
                Instruction::Gt => self.compare_op(|a, b| a > b)?,
                Instruction::Ge => self.compare_op(|a, b| a >= b)?,
                Instruction::And => {
                    let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.stack.push(RuntimeValue::Boolean(a.is_truthy() && b.is_truthy()));
                }
                Instruction::Or => {
                    let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.stack.push(RuntimeValue::Boolean(a.is_truthy() || b.is_truthy()));
                }
                Instruction::Not => {
                    let val = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    self.stack.push(RuntimeValue::Boolean(!val.is_truthy()));
                }
                Instruction::BitAnd => self.bitwise_op(|a, b| a & b)?,
                Instruction::BitOr => self.bitwise_op(|a, b| a | b)?,
                Instruction::BitXor => self.bitwise_op(|a, b| a ^ b)?,
                Instruction::BitNot => {
                    let val = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    match val {
                        RuntimeValue::Integer(n) => self.stack.push(RuntimeValue::Integer(!n)),
                        _ => return Err(RuntimeError::TypeError("Bitwise not requires integer".to_string())),
                    }
                }
                Instruction::Shl => self.bitwise_op(|a, b| a << b)?,
                Instruction::Shr => self.bitwise_op(|a, b| a >> b)?,
                Instruction::Range => {
                    let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    match (a, b) {
                        (RuntimeValue::Integer(start), RuntimeValue::Integer(end)) => {
                            let range: Vec<RuntimeValue> = (start..=end)
                                .map(RuntimeValue::Integer)
                                .collect();
                            self.stack.push(RuntimeValue::Array(range));
                        }
                        _ => return Err(RuntimeError::TypeError("Range requires integers".to_string())),
                    }
                }
                Instruction::Jump(target) => {
                    ip = *target;
                }
                Instruction::JumpIfFalse(target) => {
                    let val = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    if !val.is_truthy() {
                        ip = *target;
                    }
                }
                Instruction::JumpIfTrue(target) => {
                    let val = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    if val.is_truthy() {
                        ip = *target;
                    }
                }
                Instruction::Call(nargs) => {
                    let callee = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let mut args = Vec::new();
                    for _ in 0..*nargs {
                        args.push(self.stack.pop().unwrap_or(RuntimeValue::Nil));
                    }
                    args.reverse();

                    match callee {
                        RuntimeValue::Function(name) => {
                            let script_args: Vec<ScriptValue> = args.iter()
                                .map(|v| v.to_script_value())
                                .collect();
                            let result = ctx.call_function(&name, &script_args)?;
                            self.stack.push(RuntimeValue::from_script_value(result));
                        }
                        RuntimeValue::NativeFunction(func) => {
                            let result = func(&args)?;
                            self.stack.push(result);
                        }
                        _ => return Err(RuntimeError::NotCallable),
                    }
                }
                Instruction::Return => {
                    break;
                }
                Instruction::NewArray(size) => {
                    let mut arr = Vec::with_capacity(*size);
                    for _ in 0..*size {
                        arr.push(self.stack.pop().unwrap_or(RuntimeValue::Nil));
                    }
                    arr.reverse();
                    self.stack.push(RuntimeValue::Array(arr));
                }
                Instruction::NewTable(size) => {
                    let mut table = HashMap::new();
                    for _ in 0..*size {
                        let value = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                        let key = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                        if let RuntimeValue::String(k) = key {
                            table.insert(k, value);
                        }
                    }
                    self.stack.push(RuntimeValue::Table(table));
                }
                Instruction::GetIndex => {
                    let index = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let obj = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    
                    match (obj, index) {
                        (RuntimeValue::Array(arr), RuntimeValue::Integer(i)) => {
                            let value = arr.get(i as usize).cloned().unwrap_or(RuntimeValue::Nil);
                            self.stack.push(value);
                        }
                        (RuntimeValue::Table(table), RuntimeValue::String(key)) => {
                            let value = table.get(&key).cloned().unwrap_or(RuntimeValue::Nil);
                            self.stack.push(value);
                        }
                        (RuntimeValue::String(s), RuntimeValue::Integer(i)) => {
                            let c = s.chars().nth(i as usize)
                                .map(|c| RuntimeValue::String(c.to_string()))
                                .unwrap_or(RuntimeValue::Nil);
                            self.stack.push(c);
                        }
                        _ => self.stack.push(RuntimeValue::Nil),
                    }
                }
                Instruction::SetIndex => {
                    let value = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let index = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let obj = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    
                    match (obj, index) {
                        (RuntimeValue::Array(mut arr), RuntimeValue::Integer(i)) => {
                            if (i as usize) < arr.len() {
                                arr[i as usize] = value;
                            }
                            self.stack.push(RuntimeValue::Array(arr));
                        }
                        (RuntimeValue::Table(mut table), RuntimeValue::String(key)) => {
                            table.insert(key, value);
                            self.stack.push(RuntimeValue::Table(table));
                        }
                        _ => self.stack.push(RuntimeValue::Nil),
                    }
                }
                Instruction::GetMember(name) => {
                    let obj = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    
                    match obj {
                        RuntimeValue::Table(table) => {
                            let value = table.get(name).cloned().unwrap_or(RuntimeValue::Nil);
                            self.stack.push(value);
                        }
                        _ => self.stack.push(RuntimeValue::Nil),
                    }
                }
                Instruction::SetMember(name) => {
                    let value = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    let obj = self.stack.pop().unwrap_or(RuntimeValue::Nil);
                    
                    match obj {
                        RuntimeValue::Table(mut table) => {
                            table.insert(name.clone(), value);
                            self.stack.push(RuntimeValue::Table(table));
                        }
                        _ => self.stack.push(RuntimeValue::Nil),
                    }
                }
                Instruction::Nop => {}
            }
        }

        let elapsed = start_time.elapsed();
        let return_value = self.stack.pop().unwrap_or(RuntimeValue::Nil);

        Ok(crate::scripting::engine::ScriptResult {
            value: return_value.to_script_value(),
            execution_time_ms: elapsed.as_millis() as u64,
            instructions_executed,
            memory_used: self.stack.len() * std::mem::size_of::<RuntimeValue>(),
            output: Vec::new(),
        })
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
        let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);

        let result = match (a, b) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                RuntimeValue::Float(op(a as f64, b as f64))
            }
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                RuntimeValue::Float(op(a, b))
            }
            (RuntimeValue::Integer(a), RuntimeValue::Float(b)) => {
                RuntimeValue::Float(op(a as f64, b))
            }
            (RuntimeValue::Float(a), RuntimeValue::Integer(b)) => {
                RuntimeValue::Float(op(a, b as f64))
            }
            (RuntimeValue::String(a), RuntimeValue::String(b)) => {
                RuntimeValue::String(format!("{}{}", a, b))
            }
            _ => return Err(RuntimeError::TypeError("Invalid operand types".to_string())),
        };

        self.stack.push(result);
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
        let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);

        let result = match (a, b) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => op(a as f64, b as f64),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => op(a, b),
            (RuntimeValue::Integer(a), RuntimeValue::Float(b)) => op(a as f64, b),
            (RuntimeValue::Float(a), RuntimeValue::Integer(b)) => op(a, b as f64),
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a.cmp(&b) == std::cmp::Ordering::Less,
            _ => return Err(RuntimeError::TypeError("Cannot compare".to_string())),
        };

        self.stack.push(RuntimeValue::Boolean(result));
        Ok(())
    }

    fn bitwise_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.stack.pop().unwrap_or(RuntimeValue::Nil);
        let a = self.stack.pop().unwrap_or(RuntimeValue::Nil);

        match (a, b) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                self.stack.push(RuntimeValue::Integer(op(a, b)));
                Ok(())
            }
            _ => Err(RuntimeError::TypeError("Bitwise op requires integers".to_string())),
        }
    }

    /// Set a global variable
    pub fn set_global(&mut self, name: &str, value: RuntimeValue) {
        self.globals.insert(name.to_string(), value);
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<&RuntimeValue> {
        self.globals.get(name)
    }

    /// Clear the stack
    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }
}

/// Runtime value type
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<RuntimeValue>),
    Table(HashMap<String, RuntimeValue>),
    Function(String),
    NativeFunction(NativeFn),
    Address(u64),
    Bytes(Vec<u8>),
}

impl RuntimeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Nil => false,
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Integer(n) => *n != 0,
            RuntimeValue::Float(n) => *n != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::Table(t) => !t.is_empty(),
            _ => true,
        }
    }

    pub fn from_script_value(value: ScriptValue) -> Self {
        match value {
            ScriptValue::Nil => RuntimeValue::Nil,
            ScriptValue::Boolean(b) => RuntimeValue::Boolean(b),
            ScriptValue::Integer(n) => RuntimeValue::Integer(n),
            ScriptValue::Float(n) => RuntimeValue::Float(n),
            ScriptValue::String(s) => RuntimeValue::String(s),
            ScriptValue::Array(a) => RuntimeValue::Array(
                a.into_iter().map(RuntimeValue::from_script_value).collect()
            ),
            ScriptValue::Table(t) => RuntimeValue::Table(
                t.into_iter().map(|(k, v)| (k, RuntimeValue::from_script_value(v))).collect()
            ),
            ScriptValue::Address(addr) => RuntimeValue::Address(addr),
            ScriptValue::Bytes(b) => RuntimeValue::Bytes(b),
            ScriptValue::Function(name) => RuntimeValue::Function(name),
        }
    }

    pub fn to_script_value(&self) -> ScriptValue {
        match self {
            RuntimeValue::Nil => ScriptValue::Nil,
            RuntimeValue::Boolean(b) => ScriptValue::Boolean(*b),
            RuntimeValue::Integer(n) => ScriptValue::Integer(*n),
            RuntimeValue::Float(n) => ScriptValue::Float(*n),
            RuntimeValue::String(s) => ScriptValue::String(s.clone()),
            RuntimeValue::Array(a) => ScriptValue::Array(
                a.iter().map(|v| v.to_script_value()).collect()
            ),
            RuntimeValue::Table(t) => ScriptValue::Table(
                t.iter().map(|(k, v)| (k.clone(), v.to_script_value())).collect()
            ),
            RuntimeValue::Address(addr) => ScriptValue::Address(*addr),
            RuntimeValue::Bytes(b) => ScriptValue::Bytes(b.clone()),
            RuntimeValue::Function(name) => ScriptValue::Function(name.clone()),
            RuntimeValue::NativeFunction(_) => ScriptValue::Nil,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Nil => "nil",
            RuntimeValue::Boolean(_) => "boolean",
            RuntimeValue::Integer(_) => "integer",
            RuntimeValue::Float(_) => "float",
            RuntimeValue::String(_) => "string",
            RuntimeValue::Array(_) => "array",
            RuntimeValue::Table(_) => "table",
            RuntimeValue::Function(_) => "function",
            RuntimeValue::NativeFunction(_) => "native_function",
            RuntimeValue::Address(_) => "address",
            RuntimeValue::Bytes(_) => "bytes",
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Boolean(b) => write!(f, "{}", b),
            RuntimeValue::Integer(n) => write!(f, "{}", n),
            RuntimeValue::Float(n) => write!(f, "{}", n),
            RuntimeValue::String(s) => write!(f, "{}", s),
            RuntimeValue::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            RuntimeValue::Table(t) => {
                write!(f, "{{")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            RuntimeValue::Function(name) => write!(f, "<function {}>", name),
            RuntimeValue::NativeFunction(_) => write!(f, "<native function>"),
            RuntimeValue::Address(addr) => write!(f, "0x{:X}", addr),
            RuntimeValue::Bytes(b) => write!(f, "<{} bytes>", b.len()),
        }
    }
}

/// Native function type
pub type NativeFn = fn(&[RuntimeValue]) -> Result<RuntimeValue, RuntimeError>;

/// Runtime error types
#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError(String),
    NameError(String),
    IndexError(String),
    DivisionByZero,
    StackOverflow,
    NotCallable,
    ArgumentError(String),
    ExecutionLimitExceeded,
    MemoryLimitExceeded,
    IoError(String),
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::NameError(msg) => write!(f, "Name error: {}", msg),
            RuntimeError::IndexError(msg) => write!(f, "Index error: {}", msg),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::StackOverflow => write!(f, "Stack overflow"),
            RuntimeError::NotCallable => write!(f, "Not callable"),
            RuntimeError::ArgumentError(msg) => write!(f, "Argument error: {}", msg),
            RuntimeError::ExecutionLimitExceeded => write!(f, "Execution limit exceeded"),
            RuntimeError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            RuntimeError::IoError(msg) => write!(f, "IO error: {}", msg),
            RuntimeError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_value_truthy() {
        assert!(!RuntimeValue::Nil.is_truthy());
        assert!(RuntimeValue::Boolean(true).is_truthy());
        assert!(!RuntimeValue::Boolean(false).is_truthy());
        assert!(RuntimeValue::Integer(1).is_truthy());
        assert!(!RuntimeValue::Integer(0).is_truthy());
    }

    #[test]
    fn test_runtime_value_display() {
        assert_eq!(format!("{}", RuntimeValue::Integer(42)), "42");
        assert_eq!(format!("{}", RuntimeValue::String("hello".to_string())), "hello");
    }
}
