// Tue Jan 13 2026 - Alex

use crate::pattern::{PatternMask, Signature, PatternError};

pub struct PatternBuilder {
    name: Option<String>,
    bytes: Vec<u8>,
    mask: Vec<bool>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            bytes: Vec::new(),
            mask: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn add_byte(mut self, byte: u8) -> Self {
        self.bytes.push(byte);
        self.mask.push(true);
        self
    }

    pub fn add_wildcard(mut self) -> Self {
        self.bytes.push(0);
        self.mask.push(false);
        self
    }

    pub fn add_pattern(mut self, pattern: &str) -> Self {
        for part in pattern.split_whitespace() {
            if part == "?" || part == "??" {
                self = self.add_wildcard();
            } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                self = self.add_byte(byte);
            }
        }
        self
    }

    pub fn build_mask(self) -> PatternMask {
        PatternMask::new(self.bytes, self.mask)
    }

    pub fn build_signature(self) -> Result<Signature, PatternError> {
        let name = self.name.unwrap_or_else(|| "pattern".to_string());
        let mask = PatternMask::new(self.bytes, self.mask);
        Signature::new(name, format!("{:02x}", mask.bytes()[0]))
            .map_err(|_| PatternError::InvalidPattern("Failed to build signature".to_string()))
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}
