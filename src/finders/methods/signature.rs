// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub return_type: String,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub is_const: bool,
    pub is_static: bool,
    pub is_virtual: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

impl MethodSignature {
    pub fn new(name: &str) -> Self {
        Self {
            return_type: "void".to_string(),
            name: name.to_string(),
            parameters: Vec::new(),
            is_const: false,
            is_static: false,
            is_virtual: false,
        }
    }

    pub fn with_return_type(mut self, ret_type: &str) -> Self {
        self.return_type = ret_type.to_string();
        self
    }

    pub fn with_parameter(mut self, name: &str, param_type: &str) -> Self {
        self.parameters.push(Parameter {
            name: name.to_string(),
            param_type: param_type.to_string(),
            is_optional: false,
            default_value: None,
        });
        self
    }

    pub fn with_optional_parameter(mut self, name: &str, param_type: &str, default: &str) -> Self {
        self.parameters.push(Parameter {
            name: name.to_string(),
            param_type: param_type.to_string(),
            is_optional: true,
            default_value: Some(default.to_string()),
        });
        self
    }

    pub fn set_const(mut self, is_const: bool) -> Self {
        self.is_const = is_const;
        self
    }

    pub fn set_static(mut self, is_static: bool) -> Self {
        self.is_static = is_static;
        self
    }

    pub fn set_virtual(mut self, is_virtual: bool) -> Self {
        self.is_virtual = is_virtual;
        self
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn required_parameter_count(&self) -> usize {
        self.parameters.iter()
            .filter(|p| !p.is_optional)
            .count()
    }
}

impl fmt::Display for MethodSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_static {
            write!(f, "static ")?;
        }

        if self.is_virtual {
            write!(f, "virtual ")?;
        }

        write!(f, "{} {}(", self.return_type, self.name)?;

        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{} {}", param.param_type, param.name)?;

            if let Some(ref default) = param.default_value {
                write!(f, " = {}", default)?;
            }
        }

        write!(f, ")")?;

        if self.is_const {
            write!(f, " const")?;
        }

        Ok(())
    }
}

pub fn parse_signature(signature: &str) -> Option<MethodSignature> {
    let signature = signature.trim();

    let paren_open = signature.find('(')?;
    let paren_close = signature.rfind(')')?;

    let before_paren = &signature[..paren_open].trim();
    let params_str = &signature[paren_open + 1..paren_close].trim();

    let parts: Vec<&str> = before_paren.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let name = parts.last()?.to_string();
    let return_type = parts[..parts.len() - 1].join(" ");

    let mut sig = MethodSignature {
        return_type,
        name,
        parameters: Vec::new(),
        is_const: signature.contains(" const"),
        is_static: signature.contains("static "),
        is_virtual: signature.contains("virtual "),
    };

    if !params_str.is_empty() {
        let params: Vec<&str> = params_str.split(',').collect();

        for param in params {
            let param = param.trim();

            if param.is_empty() {
                continue;
            }

            let has_default = param.contains('=');
            let (param_decl, default_value) = if has_default {
                let eq_pos = param.find('=')?;
                (param[..eq_pos].trim(), Some(param[eq_pos + 1..].trim().to_string()))
            } else {
                (param, None)
            };

            let parts: Vec<&str> = param_decl.split_whitespace().collect();

            if parts.len() >= 2 {
                let param_name = parts.last()?.to_string();
                let param_type = parts[..parts.len() - 1].join(" ");

                sig.parameters.push(Parameter {
                    name: param_name,
                    param_type,
                    is_optional: default_value.is_some(),
                    default_value,
                });
            } else if parts.len() == 1 {
                sig.parameters.push(Parameter {
                    name: String::new(),
                    param_type: parts[0].to_string(),
                    is_optional: false,
                    default_value: None,
                });
            }
        }
    }

    Some(sig)
}
