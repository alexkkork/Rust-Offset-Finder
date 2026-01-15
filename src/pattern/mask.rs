// Tue Jan 13 2026 - Alex

pub struct PatternMask {
    bytes: Vec<u8>,
    mask: Vec<bool>,
}

impl PatternMask {
    pub fn new(bytes: Vec<u8>, mask: Vec<bool>) -> Self {
        assert_eq!(bytes.len(), mask.len());
        Self { bytes, mask }
    }

    pub fn from_pattern(pattern: &str) -> Self {
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        let mut bytes = Vec::new();
        let mut mask = Vec::new();
        for part in parts {
            if part == "?" || part == "??" {
                bytes.push(0);
                mask.push(false);
            } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                bytes.push(byte);
                mask.push(true);
            }
        }
        Self { bytes, mask }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn matches(&self, data: &[u8]) -> bool {
        if data.len() < self.bytes.len() {
            return false;
        }
        for (i, (&byte, &should_match)) in self.bytes.iter().zip(self.mask.iter()).enumerate() {
            if should_match && data[i] != byte {
                return false;
            }
        }
        true
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn mask(&self) -> &[bool] {
        &self.mask
    }
}
