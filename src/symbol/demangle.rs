// Tue Jan 13 2026 - Alex

use std::collections::VecDeque;

pub fn demangle(name: &str) -> Option<String> {
    if name.starts_with("_Z") || name.starts_with("__Z") {
        demangle_itanium(name)
    } else if name.starts_with("?") {
        demangle_msvc(name)
    } else {
        None
    }
}

pub fn demangle_itanium(mangled: &str) -> Option<String> {
    let mangled = mangled.strip_prefix("__Z")
        .or_else(|| mangled.strip_prefix("_Z"))?;

    let mut demangler = ItaniumDemangler::new(mangled);
    demangler.demangle()
}

pub fn demangle_msvc(mangled: &str) -> Option<String> {
    if !mangled.starts_with('?') {
        return None;
    }

    let mut demangler = MsvcDemangler::new(mangled);
    demangler.demangle()
}

struct ItaniumDemangler<'a> {
    input: &'a str,
    pos: usize,
    substitutions: Vec<String>,
}

impl<'a> ItaniumDemangler<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            substitutions: Vec::new(),
        }
    }

    fn demangle(&mut self) -> Option<String> {
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.peek()?;

            match c {
                'N' => {
                    self.advance();
                    result.push_str(&self.parse_nested_name()?);
                }
                'L' => {
                    self.advance();
                    continue;
                }
                '0'..='9' => {
                    let name = self.parse_source_name()?;
                    if !result.is_empty() {
                        result.push_str("::");
                    }
                    result.push_str(&name);
                }
                'S' => {
                    self.advance();
                    if let Some(sub) = self.parse_substitution() {
                        if !result.is_empty() {
                            result.push_str("::");
                        }
                        result.push_str(&sub);
                    }
                }
                'v' | 'i' | 'l' | 'x' | 'f' | 'd' | 'b' | 'c' | 's' => {
                    break;
                }
                'E' => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_nested_name(&mut self) -> Option<String> {
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.peek()?;

            match c {
                'E' => {
                    self.advance();
                    break;
                }
                '0'..='9' => {
                    let name = self.parse_source_name()?;
                    if !result.is_empty() {
                        result.push_str("::");
                    }
                    result.push_str(&name);
                }
                'S' => {
                    self.advance();
                    if let Some(sub) = self.parse_substitution() {
                        if !result.is_empty() {
                            result.push_str("::");
                        }
                        result.push_str(&sub);
                    }
                }
                'C' | 'D' => {
                    self.advance();
                    if let Some(c2) = self.peek() {
                        if c2.is_ascii_digit() {
                            self.advance();
                        }
                    }
                }
                'I' => {
                    self.advance();
                    result.push('<');
                    let mut first = true;
                    while self.peek() != Some('E') {
                        if !first {
                            result.push_str(", ");
                        }
                        first = false;
                        if let Some(arg) = self.parse_type() {
                            result.push_str(&arg);
                        } else {
                            break;
                        }
                    }
                    if self.peek() == Some('E') {
                        self.advance();
                    }
                    result.push('>');
                }
                _ => {
                    self.advance();
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn parse_source_name(&mut self) -> Option<String> {
        let mut len_str = String::new();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                len_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let len: usize = len_str.parse().ok()?;

        if self.pos + len > self.input.len() {
            return None;
        }

        let name = self.input[self.pos..self.pos + len].to_string();
        self.pos += len;

        self.substitutions.push(name.clone());

        Some(name)
    }

    fn parse_substitution(&mut self) -> Option<String> {
        let c = self.peek()?;

        match c {
            't' => {
                self.advance();
                Some("std".to_string())
            }
            'a' => {
                self.advance();
                Some("std::allocator".to_string())
            }
            'b' => {
                self.advance();
                Some("std::basic_string".to_string())
            }
            's' => {
                self.advance();
                Some("std::string".to_string())
            }
            'i' => {
                self.advance();
                Some("std::istream".to_string())
            }
            'o' => {
                self.advance();
                Some("std::ostream".to_string())
            }
            'd' => {
                self.advance();
                Some("std::iostream".to_string())
            }
            '_' => {
                self.advance();
                self.substitutions.first().cloned()
            }
            '0'..='9' | 'A'..='Z' => {
                let mut idx_str = String::new();
                while let Some(c) = self.peek() {
                    if c == '_' {
                        self.advance();
                        break;
                    }
                    if c.is_ascii_alphanumeric() {
                        idx_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let idx = parse_base36(&idx_str)?;
                self.substitutions.get(idx + 1).cloned()
            }
            _ => None,
        }
    }

    fn parse_type(&mut self) -> Option<String> {
        let c = self.peek()?;

        match c {
            'v' => { self.advance(); Some("void".to_string()) }
            'w' => { self.advance(); Some("wchar_t".to_string()) }
            'b' => { self.advance(); Some("bool".to_string()) }
            'c' => { self.advance(); Some("char".to_string()) }
            'a' => { self.advance(); Some("signed char".to_string()) }
            'h' => { self.advance(); Some("unsigned char".to_string()) }
            's' => { self.advance(); Some("short".to_string()) }
            't' => { self.advance(); Some("unsigned short".to_string()) }
            'i' => { self.advance(); Some("int".to_string()) }
            'j' => { self.advance(); Some("unsigned int".to_string()) }
            'l' => { self.advance(); Some("long".to_string()) }
            'm' => { self.advance(); Some("unsigned long".to_string()) }
            'x' => { self.advance(); Some("long long".to_string()) }
            'y' => { self.advance(); Some("unsigned long long".to_string()) }
            'f' => { self.advance(); Some("float".to_string()) }
            'd' => { self.advance(); Some("double".to_string()) }
            'e' => { self.advance(); Some("long double".to_string()) }
            'P' => {
                self.advance();
                let inner = self.parse_type()?;
                Some(format!("{}*", inner))
            }
            'R' => {
                self.advance();
                let inner = self.parse_type()?;
                Some(format!("{}&", inner))
            }
            'K' => {
                self.advance();
                let inner = self.parse_type()?;
                Some(format!("const {}", inner))
            }
            'N' => {
                self.advance();
                self.parse_nested_name()
            }
            '0'..='9' => {
                self.parse_source_name()
            }
            'S' => {
                self.advance();
                self.parse_substitution()
            }
            _ => {
                self.advance();
                None
            }
        }
    }
}

fn parse_base36(s: &str) -> Option<usize> {
    let mut result = 0usize;
    for c in s.chars() {
        let digit = if c.is_ascii_digit() {
            c as usize - '0' as usize
        } else if c.is_ascii_uppercase() {
            c as usize - 'A' as usize + 10
        } else {
            return None;
        };
        result = result * 36 + digit;
    }
    Some(result)
}

struct MsvcDemangler<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> MsvcDemangler<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn demangle(&mut self) -> Option<String> {
        if !self.input.starts_with('?') {
            return None;
        }
        self.pos = 1;

        let name = self.parse_qualified_name()?;

        Some(name)
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_qualified_name(&mut self) -> Option<String> {
        let mut parts = VecDeque::new();

        while self.pos < self.input.len() {
            let c = self.peek()?;

            match c {
                '@' => {
                    self.advance();
                    if self.peek() == Some('@') {
                        self.advance();
                        break;
                    }
                }
                '?' => {
                    self.advance();
                    if let Some(special) = self.parse_special_name() {
                        parts.push_front(special);
                    }
                }
                _ => {
                    if let Some(name) = self.parse_simple_name() {
                        parts.push_front(name);
                    } else {
                        break;
                    }
                }
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.into_iter().collect::<Vec<_>>().join("::"))
        }
    }

    fn parse_simple_name(&mut self) -> Option<String> {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c == '@' {
                let name = self.input[start..self.pos].to_string();
                return Some(name);
            }
            self.advance();
        }

        None
    }

    fn parse_special_name(&mut self) -> Option<String> {
        let c = self.peek()?;

        match c {
            '0' => { self.advance(); Some("~destructor".to_string()) }
            '1' => { self.advance(); Some("operator new".to_string()) }
            '2' => { self.advance(); Some("operator delete".to_string()) }
            '3' => { self.advance(); Some("operator=".to_string()) }
            '4' => { self.advance(); Some("operator>>".to_string()) }
            '5' => { self.advance(); Some("operator<<".to_string()) }
            '6' => { self.advance(); Some("operator!".to_string()) }
            '7' => { self.advance(); Some("operator==".to_string()) }
            '8' => { self.advance(); Some("operator!=".to_string()) }
            '9' => { self.advance(); Some("operator[]".to_string()) }
            'A' => { self.advance(); Some("operator->".to_string()) }
            'B' => { self.advance(); Some("operator*".to_string()) }
            'C' => { self.advance(); Some("operator++".to_string()) }
            'D' => { self.advance(); Some("operator--".to_string()) }
            _ => None,
        }
    }
}

pub fn is_mangled(name: &str) -> bool {
    name.starts_with("_Z")
        || name.starts_with("__Z")
        || name.starts_with("?")
}

pub fn try_demangle(name: &str) -> String {
    demangle(name).unwrap_or_else(|| name.to_string())
}
