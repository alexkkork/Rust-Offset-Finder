// Tue Jan 15 2026 - Alex

use std::collections::HashMap;
use std::fmt;

/// Script value types
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Table(HashMap<String, ScriptValue>),
    Address(u64),
    Bytes(Vec<u8>),
    Function(String),
}

impl ScriptValue {
    pub fn nil() -> Self {
        ScriptValue::Nil
    }

    pub fn boolean(b: bool) -> Self {
        ScriptValue::Boolean(b)
    }

    pub fn integer(n: i64) -> Self {
        ScriptValue::Integer(n)
    }

    pub fn float(n: f64) -> Self {
        ScriptValue::Float(n)
    }

    pub fn string(s: impl Into<String>) -> Self {
        ScriptValue::String(s.into())
    }

    pub fn array(items: Vec<ScriptValue>) -> Self {
        ScriptValue::Array(items)
    }

    pub fn table(items: HashMap<String, ScriptValue>) -> Self {
        ScriptValue::Table(items)
    }

    pub fn address(addr: u64) -> Self {
        ScriptValue::Address(addr)
    }

    pub fn bytes(data: Vec<u8>) -> Self {
        ScriptValue::Bytes(data)
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, ScriptValue::Nil)
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            ScriptValue::Nil => false,
            ScriptValue::Boolean(b) => *b,
            ScriptValue::Integer(n) => *n != 0,
            ScriptValue::Float(n) => *n != 0.0,
            ScriptValue::String(s) => !s.is_empty(),
            ScriptValue::Array(a) => !a.is_empty(),
            ScriptValue::Table(t) => !t.is_empty(),
            _ => true,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ScriptValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            ScriptValue::Integer(n) => Some(*n),
            ScriptValue::Float(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ScriptValue::Float(n) => Some(*n),
            ScriptValue::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            ScriptValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ScriptValue>> {
        match self {
            ScriptValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&HashMap<String, ScriptValue>> {
        match self {
            ScriptValue::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_address(&self) -> Option<u64> {
        match self {
            ScriptValue::Address(a) => Some(*a),
            ScriptValue::Integer(n) => Some(*n as u64),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            ScriptValue::Nil => "nil",
            ScriptValue::Boolean(_) => "boolean",
            ScriptValue::Integer(_) => "integer",
            ScriptValue::Float(_) => "float",
            ScriptValue::String(_) => "string",
            ScriptValue::Array(_) => "array",
            ScriptValue::Table(_) => "table",
            ScriptValue::Address(_) => "address",
            ScriptValue::Bytes(_) => "bytes",
            ScriptValue::Function(_) => "function",
        }
    }

    pub fn memory_size(&self) -> usize {
        match self {
            ScriptValue::Nil | ScriptValue::Boolean(_) => std::mem::size_of::<Self>(),
            ScriptValue::Integer(_) | ScriptValue::Float(_) => std::mem::size_of::<Self>(),
            ScriptValue::String(s) => std::mem::size_of::<Self>() + s.len(),
            ScriptValue::Array(a) => {
                std::mem::size_of::<Self>() + a.iter().map(|v| v.memory_size()).sum::<usize>()
            }
            ScriptValue::Table(t) => {
                std::mem::size_of::<Self>() + 
                t.iter().map(|(k, v)| k.len() + v.memory_size()).sum::<usize>()
            }
            ScriptValue::Address(_) => std::mem::size_of::<Self>(),
            ScriptValue::Bytes(b) => std::mem::size_of::<Self>() + b.len(),
            ScriptValue::Function(name) => std::mem::size_of::<Self>() + name.len(),
        }
    }
}

impl Default for ScriptValue {
    fn default() -> Self {
        ScriptValue::Nil
    }
}

impl fmt::Display for ScriptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptValue::Nil => write!(f, "nil"),
            ScriptValue::Boolean(b) => write!(f, "{}", b),
            ScriptValue::Integer(n) => write!(f, "{}", n),
            ScriptValue::Float(n) => write!(f, "{}", n),
            ScriptValue::String(s) => write!(f, "{}", s),
            ScriptValue::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            ScriptValue::Table(t) => {
                write!(f, "{{")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            ScriptValue::Address(addr) => write!(f, "0x{:X}", addr),
            ScriptValue::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            ScriptValue::Function(name) => write!(f, "<function {}>", name),
        }
    }
}

/// Script type definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptType {
    Nil,
    Boolean,
    Integer,
    Float,
    String,
    Array(Box<ScriptType>),
    Table,
    Address,
    Bytes,
    Function(Vec<ScriptType>, Box<ScriptType>),
    Any,
    Union(Vec<ScriptType>),
}

impl ScriptType {
    pub fn is_compatible(&self, other: &ScriptType) -> bool {
        if self == other || *self == ScriptType::Any || *other == ScriptType::Any {
            return true;
        }

        match (self, other) {
            (ScriptType::Union(types), t) | (t, ScriptType::Union(types)) => {
                types.iter().any(|u| u.is_compatible(t))
            }
            (ScriptType::Array(a), ScriptType::Array(b)) => a.is_compatible(b),
            (ScriptType::Integer, ScriptType::Float) | 
            (ScriptType::Float, ScriptType::Integer) => true,
            _ => false,
        }
    }

    pub fn name(&self) -> String {
        match self {
            ScriptType::Nil => "nil".to_string(),
            ScriptType::Boolean => "boolean".to_string(),
            ScriptType::Integer => "integer".to_string(),
            ScriptType::Float => "float".to_string(),
            ScriptType::String => "string".to_string(),
            ScriptType::Array(t) => format!("array<{}>", t.name()),
            ScriptType::Table => "table".to_string(),
            ScriptType::Address => "address".to_string(),
            ScriptType::Bytes => "bytes".to_string(),
            ScriptType::Function(params, ret) => {
                let params_str = params.iter()
                    .map(|p| p.name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params_str, ret.name())
            }
            ScriptType::Any => "any".to_string(),
            ScriptType::Union(types) => {
                types.iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        }
    }
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Script function definition
#[derive(Debug, Clone)]
pub struct ScriptFunction {
    pub name: String,
    pub params: Vec<(String, ScriptType)>,
    pub return_type: ScriptType,
    pub body: Vec<u8>,
    pub is_native: bool,
}

impl ScriptFunction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            params: Vec::new(),
            return_type: ScriptType::Any,
            body: Vec::new(),
            is_native: false,
        }
    }

    pub fn with_param(mut self, name: &str, typ: ScriptType) -> Self {
        self.params.push((name.to_string(), typ));
        self
    }

    pub fn with_return_type(mut self, typ: ScriptType) -> Self {
        self.return_type = typ;
        self
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_value_truthy() {
        assert!(!ScriptValue::Nil.is_truthy());
        assert!(ScriptValue::Boolean(true).is_truthy());
        assert!(ScriptValue::Integer(1).is_truthy());
        assert!(!ScriptValue::Integer(0).is_truthy());
    }

    #[test]
    fn test_script_value_conversions() {
        assert_eq!(ScriptValue::Integer(42).as_int(), Some(42));
        assert_eq!(ScriptValue::Float(3.14).as_float(), Some(3.14));
        assert_eq!(ScriptValue::String("hello".to_string()).as_str(), Some("hello"));
    }

    #[test]
    fn test_script_type_compatibility() {
        assert!(ScriptType::Integer.is_compatible(&ScriptType::Float));
        assert!(ScriptType::Any.is_compatible(&ScriptType::String));
    }
}
