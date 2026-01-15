// Tue Jan 13 2026 - Alex

use crate::pattern::{PatternMask, PatternError};

pub struct BytecodePattern {
    instructions: Vec<u32>,
    mask: PatternMask,
}

impl BytecodePattern {
    pub fn from_instructions(instructions: Vec<u32>) -> Result<Self, PatternError> {
        let mut bytes = Vec::new();
        for inst in &instructions {
            bytes.extend_from_slice(&inst.to_le_bytes());
        }
        let mask = vec![true; bytes.len()];
        Ok(Self {
            instructions,
            mask: PatternMask::new(bytes, mask),
        })
    }

    pub fn from_bytes(bytes: Vec<u8>, mask: Vec<bool>) -> Self {
        Self {
            instructions: Vec::new(),
            mask: PatternMask::new(bytes, mask),
        }
    }

    pub fn mask(&self) -> &PatternMask {
        &self.mask
    }

    pub fn instructions(&self) -> &[u32] {
        &self.instructions
    }

    pub fn len(&self) -> usize {
        self.mask.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mask.is_empty()
    }
}
