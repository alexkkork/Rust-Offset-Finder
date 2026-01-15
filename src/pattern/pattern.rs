// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone)]
pub struct Pattern {
    bytes: Vec<u8>,
    mask: Vec<bool>,
    name: Option<String>,
}

impl Pattern {
    pub fn new(bytes: Vec<u8>, mask: Vec<bool>) -> Self {
        assert_eq!(bytes.len(), mask.len(), "Pattern bytes and mask must have same length");
        Self {
            bytes,
            mask,
            name: None,
        }
    }

    pub fn from_hex(hex: &str) -> Self {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();

        for part in hex.split_whitespace() {
            if part == "??" || part == "?" {
                bytes.push(0);
                mask.push(false);
            } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                bytes.push(byte);
                mask.push(true);
            }
        }

        Self {
            bytes,
            mask,
            name: None,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mask = vec![true; bytes.len()];
        Self {
            bytes: bytes.to_vec(),
            mask,
            name: None,
        }
    }

    pub fn from_ida_pattern(pattern: &str) -> Self {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();

        let mut chars = pattern.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                continue;
            }

            if c == '?' {
                bytes.push(0);
                mask.push(false);
                if chars.peek() == Some(&'?') {
                    chars.next();
                }
            } else if c.is_ascii_hexdigit() {
                let mut hex = String::new();
                hex.push(c);
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_hexdigit() {
                        hex.push(chars.next().unwrap());
                    }
                }
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                    mask.push(true);
                }
            }
        }

        Self {
            bytes,
            mask,
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn mask(&self) -> &[bool] {
        &self.mask
    }

    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < self.bytes.len() {
            return false;
        }

        self.bytes.iter()
            .zip(self.mask.iter())
            .zip(data.iter())
            .all(|((pattern_byte, &significant), &data_byte)| {
                !significant || *pattern_byte == data_byte
            })
    }

    pub fn find_in(&self, data: &[u8]) -> Option<usize> {
        if self.bytes.is_empty() || data.len() < self.bytes.len() {
            return None;
        }

        let first_significant = self.mask.iter()
            .position(|&m| m)
            .unwrap_or(0);

        let first_byte = self.bytes[first_significant];

        for i in 0..=(data.len() - self.bytes.len()) {
            if data[i + first_significant] == first_byte && self.matches(&data[i..]) {
                return Some(i);
            }
        }

        None
    }

    pub fn find_all_in(&self, data: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();

        if self.bytes.is_empty() || data.len() < self.bytes.len() {
            return results;
        }

        let first_significant = self.mask.iter()
            .position(|&m| m)
            .unwrap_or(0);

        let first_byte = self.bytes[first_significant];

        for i in 0..=(data.len() - self.bytes.len()) {
            if data[i + first_significant] == first_byte && self.matches(&data[i..]) {
                results.push(i);
            }
        }

        results
    }

    pub fn significant_byte_count(&self) -> usize {
        self.mask.iter().filter(|&&m| m).count()
    }

    pub fn wildcard_byte_count(&self) -> usize {
        self.mask.iter().filter(|&&m| !m).count()
    }

    pub fn to_hex_string(&self) -> String {
        self.bytes.iter()
            .zip(self.mask.iter())
            .map(|(b, &m)| {
                if m {
                    format!("{:02X}", b)
                } else {
                    "??".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}: ", name)?;
        }
        write!(f, "{}", self.to_hex_string())
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes && self.mask == other.mask
    }
}

impl Eq for Pattern {}

pub struct PatternBuilder {
    bytes: Vec<u8>,
    mask: Vec<bool>,
    name: Option<String>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            mask: Vec::new(),
            name: None,
        }
    }

    pub fn byte(mut self, b: u8) -> Self {
        self.bytes.push(b);
        self.mask.push(true);
        self
    }

    pub fn bytes(mut self, bs: &[u8]) -> Self {
        for &b in bs {
            self.bytes.push(b);
            self.mask.push(true);
        }
        self
    }

    pub fn wildcard(mut self) -> Self {
        self.bytes.push(0);
        self.mask.push(false);
        self
    }

    pub fn wildcards(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.bytes.push(0);
            self.mask.push(false);
        }
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn build(self) -> Pattern {
        Pattern {
            bytes: self.bytes,
            mask: self.mask,
            name: self.name,
        }
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern {
    /// Create a pattern with a byte mask (0xFF = fixed, 0x00 = wildcard)
    pub fn with_mask(bytes: &[u8], byte_mask: &[u8]) -> Self {
        let mask: Vec<bool> = byte_mask.iter().map(|&m| m == 0xFF).collect();
        Self {
            bytes: bytes.to_vec(),
            mask,
            name: None,
        }
    }

    /// Get the mask as bytes (0xFF for fixed, 0x00 for wildcard)
    pub fn mask_as_bytes(&self) -> Vec<u8> {
        self.mask.iter().map(|&m| if m { 0xFF } else { 0x00 }).collect()
    }
}
