// Tue Jan 15 2026 - Alex

use crate::scripting::api::{ScriptApi, ApiModule, ApiModuleBuilder};
use crate::scripting::types::ScriptValue;
use crate::scripting::runtime::RuntimeError;

/// Register all built-in functions
pub fn register_builtins(api: &mut ScriptApi) {
    // Core functions
    register_core_functions(api);
    
    // Memory module
    api.register_module(create_memory_module());
    
    // Math module
    api.register_module(create_math_module());
    
    // String module
    api.register_module(create_string_module());
    
    // Array module
    api.register_module(create_array_module());
}

/// Built-in functions registry
pub struct BuiltinFunctions;

impl BuiltinFunctions {
    pub fn list() -> Vec<(&'static str, &'static str)> {
        vec![
            ("print", "Print values to output"),
            ("type", "Get type name of a value"),
            ("len", "Get length of array/string"),
            ("tostring", "Convert value to string"),
            ("toint", "Convert value to integer"),
            ("tofloat", "Convert value to float"),
            ("hex", "Convert integer to hex string"),
            ("address", "Create an address from integer"),
            ("range", "Create a range array"),
            ("memory.read_u8", "Read unsigned 8-bit value"),
            ("memory.read_u16", "Read unsigned 16-bit value"),
            ("memory.read_u32", "Read unsigned 32-bit value"),
            ("memory.read_u64", "Read unsigned 64-bit value"),
            ("memory.read_i8", "Read signed 8-bit value"),
            ("memory.read_i16", "Read signed 16-bit value"),
            ("memory.read_i32", "Read signed 32-bit value"),
            ("memory.read_i64", "Read signed 64-bit value"),
            ("memory.read_f32", "Read 32-bit float"),
            ("memory.read_f64", "Read 64-bit float"),
            ("memory.read_string", "Read null-terminated string"),
            ("memory.read_bytes", "Read raw bytes"),
            ("math.abs", "Absolute value"),
            ("math.min", "Minimum value"),
            ("math.max", "Maximum value"),
            ("math.floor", "Floor of float"),
            ("math.ceil", "Ceiling of float"),
            ("math.round", "Round float"),
            ("math.sqrt", "Square root"),
            ("math.pow", "Power function"),
            ("string.len", "String length"),
            ("string.upper", "Convert to uppercase"),
            ("string.lower", "Convert to lowercase"),
            ("string.substr", "Get substring"),
            ("string.find", "Find substring"),
            ("string.split", "Split string"),
            ("string.trim", "Trim whitespace"),
            ("array.len", "Array length"),
            ("array.push", "Push to array"),
            ("array.pop", "Pop from array"),
            ("array.join", "Join array elements"),
            ("array.reverse", "Reverse array"),
            ("array.sort", "Sort array"),
            ("array.filter", "Filter array"),
            ("array.map", "Map over array"),
        ]
    }
}

fn register_core_functions(api: &mut ScriptApi) {
    // print
    api.register_function("print", |args| {
        let output: Vec<String> = args.iter()
            .map(|v| format!("{}", v))
            .collect();
        println!("{}", output.join(" "));
        Ok(ScriptValue::Nil)
    });

    // type
    api.register_function("type", |args| {
        if let Some(v) = args.first() {
            Ok(ScriptValue::String(v.type_name().to_string()))
        } else {
            Ok(ScriptValue::String("nil".to_string()))
        }
    });

    // len
    api.register_function("len", |args| {
        if let Some(v) = args.first() {
            let len = match v {
                ScriptValue::String(s) => s.len(),
                ScriptValue::Array(a) => a.len(),
                ScriptValue::Table(t) => t.len(),
                ScriptValue::Bytes(b) => b.len(),
                _ => return Err(RuntimeError::TypeError("Cannot get length".to_string())),
            };
            Ok(ScriptValue::Integer(len as i64))
        } else {
            Ok(ScriptValue::Integer(0))
        }
    });

    // tostring
    api.register_function("tostring", |args| {
        if let Some(v) = args.first() {
            Ok(ScriptValue::String(format!("{}", v)))
        } else {
            Ok(ScriptValue::String("nil".to_string()))
        }
    });

    // toint
    api.register_function("toint", |args| {
        if let Some(v) = args.first() {
            let n = match v {
                ScriptValue::Integer(n) => *n,
                ScriptValue::Float(n) => *n as i64,
                ScriptValue::String(s) => {
                    if s.starts_with("0x") || s.starts_with("0X") {
                        i64::from_str_radix(&s[2..], 16).unwrap_or(0)
                    } else {
                        s.parse().unwrap_or(0)
                    }
                }
                ScriptValue::Boolean(b) => if *b { 1 } else { 0 },
                _ => 0,
            };
            Ok(ScriptValue::Integer(n))
        } else {
            Ok(ScriptValue::Integer(0))
        }
    });

    // tofloat
    api.register_function("tofloat", |args| {
        if let Some(v) = args.first() {
            let n = match v {
                ScriptValue::Integer(n) => *n as f64,
                ScriptValue::Float(n) => *n,
                ScriptValue::String(s) => s.parse().unwrap_or(0.0),
                _ => 0.0,
            };
            Ok(ScriptValue::Float(n))
        } else {
            Ok(ScriptValue::Float(0.0))
        }
    });

    // hex
    api.register_function("hex", |args| {
        if let Some(v) = args.first() {
            let s = match v {
                ScriptValue::Integer(n) => format!("0x{:X}", n),
                ScriptValue::Address(a) => format!("0x{:X}", a),
                _ => return Err(RuntimeError::TypeError("Cannot convert to hex".to_string())),
            };
            Ok(ScriptValue::String(s))
        } else {
            Ok(ScriptValue::String("0x0".to_string()))
        }
    });

    // address
    api.register_function("address", |args| {
        if let Some(v) = args.first() {
            let addr = match v {
                ScriptValue::Integer(n) => *n as u64,
                ScriptValue::Address(a) => *a,
                ScriptValue::String(s) => {
                    if s.starts_with("0x") || s.starts_with("0X") {
                        u64::from_str_radix(&s[2..], 16).unwrap_or(0)
                    } else {
                        s.parse().unwrap_or(0)
                    }
                }
                _ => 0,
            };
            Ok(ScriptValue::Address(addr))
        } else {
            Ok(ScriptValue::Address(0))
        }
    });

    // range
    api.register_function("range", |args| {
        let (start, end, step) = match args.len() {
            1 => (0i64, args[0].as_int().unwrap_or(0), 1i64),
            2 => (args[0].as_int().unwrap_or(0), args[1].as_int().unwrap_or(0), 1i64),
            _ => (
                args[0].as_int().unwrap_or(0),
                args[1].as_int().unwrap_or(0),
                args.get(2).and_then(|v| v.as_int()).unwrap_or(1)
            ),
        };

        if step == 0 {
            return Err(RuntimeError::ArgumentError("Step cannot be zero".to_string()));
        }

        let mut result = Vec::new();
        let mut i = start;
        
        if step > 0 {
            while i < end {
                result.push(ScriptValue::Integer(i));
                i += step;
            }
        } else {
            while i > end {
                result.push(ScriptValue::Integer(i));
                i += step;
            }
        }

        Ok(ScriptValue::Array(result))
    });
}

fn create_memory_module() -> ApiModule {
    ApiModuleBuilder::new("memory")
        .description("Memory reading functions")
        .function("read_u8", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            // In a real implementation, would use the reader
            // For now, return placeholder
            Ok(ScriptValue::Integer(0))
        })
        .function("read_u16", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_u32", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_u64", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_i8", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_i16", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_i32", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_i64", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Integer(0))
        })
        .function("read_f32", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Float(0.0))
        })
        .function("read_f64", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            Ok(ScriptValue::Float(0.0))
        })
        .function("read_string", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            let _max_len = args.get(1).and_then(|v| v.as_int()).unwrap_or(256) as usize;
            Ok(ScriptValue::String(String::new()))
        })
        .function("read_bytes", |args| {
            let addr = args.get(0)
                .and_then(|v| v.as_address())
                .ok_or_else(|| RuntimeError::ArgumentError("Address required".to_string()))?;
            let len = args.get(1)
                .and_then(|v| v.as_int())
                .ok_or_else(|| RuntimeError::ArgumentError("Length required".to_string()))? as usize;
            Ok(ScriptValue::Bytes(vec![0; len]))
        })
        .build()
}

fn create_math_module() -> ApiModule {
    ApiModuleBuilder::new("math")
        .description("Mathematical functions")
        .function("abs", |args| {
            if let Some(v) = args.first() {
                match v {
                    ScriptValue::Integer(n) => Ok(ScriptValue::Integer(n.abs())),
                    ScriptValue::Float(n) => Ok(ScriptValue::Float(n.abs())),
                    _ => Err(RuntimeError::TypeError("Number required".to_string())),
                }
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("min", |args| {
            if args.is_empty() {
                return Ok(ScriptValue::Nil);
            }
            
            let mut min = args[0].clone();
            for v in &args[1..] {
                let cmp = match (&min, v) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => *a > *b,
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => *a > *b,
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => (*a as f64) > *b,
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => *a > (*b as f64),
                    _ => false,
                };
                if cmp {
                    min = v.clone();
                }
            }
            Ok(min)
        })
        .function("max", |args| {
            if args.is_empty() {
                return Ok(ScriptValue::Nil);
            }
            
            let mut max = args[0].clone();
            for v in &args[1..] {
                let cmp = match (&max, v) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => *a < *b,
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => *a < *b,
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => (*a as f64) < *b,
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => *a < (*b as f64),
                    _ => false,
                };
                if cmp {
                    max = v.clone();
                }
            }
            Ok(max)
        })
        .function("floor", |args| {
            if let Some(v) = args.first() {
                match v {
                    ScriptValue::Float(n) => Ok(ScriptValue::Integer(n.floor() as i64)),
                    ScriptValue::Integer(n) => Ok(ScriptValue::Integer(*n)),
                    _ => Err(RuntimeError::TypeError("Number required".to_string())),
                }
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("ceil", |args| {
            if let Some(v) = args.first() {
                match v {
                    ScriptValue::Float(n) => Ok(ScriptValue::Integer(n.ceil() as i64)),
                    ScriptValue::Integer(n) => Ok(ScriptValue::Integer(*n)),
                    _ => Err(RuntimeError::TypeError("Number required".to_string())),
                }
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("round", |args| {
            if let Some(v) = args.first() {
                match v {
                    ScriptValue::Float(n) => Ok(ScriptValue::Integer(n.round() as i64)),
                    ScriptValue::Integer(n) => Ok(ScriptValue::Integer(*n)),
                    _ => Err(RuntimeError::TypeError("Number required".to_string())),
                }
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("sqrt", |args| {
            if let Some(v) = args.first() {
                let n = match v {
                    ScriptValue::Float(n) => *n,
                    ScriptValue::Integer(n) => *n as f64,
                    _ => return Err(RuntimeError::TypeError("Number required".to_string())),
                };
                Ok(ScriptValue::Float(n.sqrt()))
            } else {
                Ok(ScriptValue::Float(0.0))
            }
        })
        .function("pow", |args| {
            let base = args.get(0).and_then(|v| v.as_float()).unwrap_or(0.0);
            let exp = args.get(1).and_then(|v| v.as_float()).unwrap_or(1.0);
            Ok(ScriptValue::Float(base.powf(exp)))
        })
        .build()
}

fn create_string_module() -> ApiModule {
    ApiModuleBuilder::new("string")
        .description("String manipulation functions")
        .function("len", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                Ok(ScriptValue::Integer(s.len() as i64))
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("upper", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                Ok(ScriptValue::String(s.to_uppercase()))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .function("lower", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                Ok(ScriptValue::String(s.to_lowercase()))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .function("substr", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                let start = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as usize;
                let len = args.get(2).and_then(|v| v.as_int()).map(|n| n as usize);
                
                let end = len.map(|l| start + l).unwrap_or(s.len());
                let substr: String = s.chars().skip(start).take(end - start).collect();
                Ok(ScriptValue::String(substr))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .function("find", |args| {
            if let (Some(ScriptValue::String(haystack)), Some(ScriptValue::String(needle))) = 
                (args.get(0), args.get(1)) {
                match haystack.find(needle) {
                    Some(idx) => Ok(ScriptValue::Integer(idx as i64)),
                    None => Ok(ScriptValue::Integer(-1)),
                }
            } else {
                Ok(ScriptValue::Integer(-1))
            }
        })
        .function("split", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                let sep = args.get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or(" ");
                
                let parts: Vec<ScriptValue> = s.split(sep)
                    .map(|p| ScriptValue::String(p.to_string()))
                    .collect();
                Ok(ScriptValue::Array(parts))
            } else {
                Ok(ScriptValue::Array(Vec::new()))
            }
        })
        .function("trim", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                Ok(ScriptValue::String(s.trim().to_string()))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .function("starts_with", |args| {
            if let (Some(ScriptValue::String(s)), Some(ScriptValue::String(prefix))) = 
                (args.get(0), args.get(1)) {
                Ok(ScriptValue::Boolean(s.starts_with(prefix)))
            } else {
                Ok(ScriptValue::Boolean(false))
            }
        })
        .function("ends_with", |args| {
            if let (Some(ScriptValue::String(s)), Some(ScriptValue::String(suffix))) = 
                (args.get(0), args.get(1)) {
                Ok(ScriptValue::Boolean(s.ends_with(suffix)))
            } else {
                Ok(ScriptValue::Boolean(false))
            }
        })
        .function("replace", |args| {
            if let Some(ScriptValue::String(s)) = args.first() {
                let from = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
                let to = args.get(2).and_then(|v| v.as_str()).unwrap_or("");
                Ok(ScriptValue::String(s.replace(from, to)))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .build()
}

fn create_array_module() -> ApiModule {
    ApiModuleBuilder::new("array")
        .description("Array manipulation functions")
        .function("len", |args| {
            if let Some(ScriptValue::Array(a)) = args.first() {
                Ok(ScriptValue::Integer(a.len() as i64))
            } else {
                Ok(ScriptValue::Integer(0))
            }
        })
        .function("push", |args| {
            if let Some(ScriptValue::Array(mut arr)) = args.first().cloned() {
                if let Some(value) = args.get(1) {
                    arr.push(value.clone());
                }
                Ok(ScriptValue::Array(arr))
            } else {
                Ok(ScriptValue::Array(Vec::new()))
            }
        })
        .function("pop", |args| {
            if let Some(ScriptValue::Array(mut arr)) = args.first().cloned() {
                let popped = arr.pop().unwrap_or(ScriptValue::Nil);
                Ok(popped)
            } else {
                Ok(ScriptValue::Nil)
            }
        })
        .function("join", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                let sep = args.get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or(",");
                
                let joined: String = arr.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(sep);
                Ok(ScriptValue::String(joined))
            } else {
                Ok(ScriptValue::String(String::new()))
            }
        })
        .function("reverse", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(ScriptValue::Array(reversed))
            } else {
                Ok(ScriptValue::Array(Vec::new()))
            }
        })
        .function("sort", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                let mut sorted = arr.clone();
                // Simple sort by converting to strings
                sorted.sort_by(|a, b| format!("{}", a).cmp(&format!("{}", b)));
                Ok(ScriptValue::Array(sorted))
            } else {
                Ok(ScriptValue::Array(Vec::new()))
            }
        })
        .function("contains", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                if let Some(value) = args.get(1) {
                    Ok(ScriptValue::Boolean(arr.contains(value)))
                } else {
                    Ok(ScriptValue::Boolean(false))
                }
            } else {
                Ok(ScriptValue::Boolean(false))
            }
        })
        .function("first", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                Ok(arr.first().cloned().unwrap_or(ScriptValue::Nil))
            } else {
                Ok(ScriptValue::Nil)
            }
        })
        .function("last", |args| {
            if let Some(ScriptValue::Array(arr)) = args.first() {
                Ok(arr.last().cloned().unwrap_or(ScriptValue::Nil))
            } else {
                Ok(ScriptValue::Nil)
            }
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_list() {
        let list = BuiltinFunctions::list();
        assert!(!list.is_empty());
        assert!(list.iter().any(|(name, _)| *name == "print"));
    }
}
