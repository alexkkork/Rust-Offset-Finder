// Tue Jan 13 2026 - Alex

use crate::pattern::Pattern;
use std::collections::HashMap;

pub struct PatternCompiler {
    optimizations: bool,
}

impl PatternCompiler {
    pub fn new() -> Self {
        Self {
            optimizations: true,
        }
    }

    pub fn set_optimizations(mut self, enabled: bool) -> Self {
        self.optimizations = enabled;
        self
    }

    pub fn compile(&self, source: &str) -> Result<CompiledPattern, CompileError> {
        let pattern = self.parse(source)?;

        let first_significant = pattern.mask().iter()
            .position(|&m| m)
            .ok_or(CompileError::NoSignificantBytes)?;

        let skip_table = if self.optimizations {
            Some(self.build_skip_table(&pattern))
        } else {
            None
        };

        Ok(CompiledPattern {
            pattern,
            first_significant,
            skip_table,
        })
    }

    fn parse(&self, source: &str) -> Result<Pattern, CompileError> {
        let source = source.trim();

        if source.is_empty() {
            return Err(CompileError::EmptyPattern);
        }

        let mut bytes = Vec::new();
        let mut mask = Vec::new();

        for token in source.split_whitespace() {
            match token {
                "?" | "??" | "x" | "X" => {
                    bytes.push(0);
                    mask.push(false);
                }
                _ => {
                    let byte = u8::from_str_radix(token, 16)
                        .map_err(|_| CompileError::InvalidByte(token.to_string()))?;
                    bytes.push(byte);
                    mask.push(true);
                }
            }
        }

        if bytes.is_empty() {
            return Err(CompileError::EmptyPattern);
        }

        Ok(Pattern::new(bytes, mask))
    }

    fn build_skip_table(&self, pattern: &Pattern) -> HashMap<u8, usize> {
        let mut table = HashMap::new();
        let len = pattern.len();

        for i in 0..=255u8 {
            table.insert(i, len);
        }

        for i in 0..len - 1 {
            if pattern.mask()[i] {
                table.insert(pattern.bytes()[i], len - 1 - i);
            }
        }

        table
    }
}

impl Default for PatternCompiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompiledPattern {
    pattern: Pattern,
    first_significant: usize,
    skip_table: Option<HashMap<u8, usize>>,
}

impl CompiledPattern {
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    pub fn find_in(&self, data: &[u8]) -> Option<usize> {
        if self.pattern.is_empty() || data.len() < self.pattern.len() {
            return None;
        }

        if let Some(ref skip_table) = self.skip_table {
            self.find_boyer_moore(data, skip_table)
        } else {
            self.find_naive(data)
        }
    }

    fn find_naive(&self, data: &[u8]) -> Option<usize> {
        let first_byte = self.pattern.bytes()[self.first_significant];

        for i in 0..=(data.len() - self.pattern.len()) {
            if data[i + self.first_significant] == first_byte && self.pattern.matches(&data[i..]) {
                return Some(i);
            }
        }

        None
    }

    fn find_boyer_moore(&self, data: &[u8], skip_table: &HashMap<u8, usize>) -> Option<usize> {
        let pattern_len = self.pattern.len();
        let mut i = 0;

        while i <= data.len() - pattern_len {
            let mut j = pattern_len - 1;

            while self.pattern.mask()[j] == false || data[i + j] == self.pattern.bytes()[j] {
                if j == 0 {
                    return Some(i);
                }
                j -= 1;
            }

            let skip = skip_table.get(&data[i + pattern_len - 1]).copied().unwrap_or(pattern_len);
            i += skip.max(1);
        }

        None
    }

    pub fn find_all_in(&self, data: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();

        if self.pattern.is_empty() || data.len() < self.pattern.len() {
            return results;
        }

        let first_byte = self.pattern.bytes()[self.first_significant];

        for i in 0..=(data.len() - self.pattern.len()) {
            if data[i + self.first_significant] == first_byte && self.pattern.matches(&data[i..]) {
                results.push(i);
            }
        }

        results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    EmptyPattern,
    InvalidByte(String),
    NoSignificantBytes,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::EmptyPattern => write!(f, "Pattern is empty"),
            CompileError::InvalidByte(s) => write!(f, "Invalid byte in pattern: {}", s),
            CompileError::NoSignificantBytes => write!(f, "Pattern has no significant bytes"),
        }
    }
}

impl std::error::Error for CompileError {}
