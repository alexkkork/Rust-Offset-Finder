// Tue Jan 13 2026 - Alex

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wildcard {
    Any,
    Byte(u8),
}

impl Wildcard {
    pub fn matches(&self, byte: u8) -> bool {
        match self {
            Self::Any => true,
            Self::Byte(b) => *b == byte,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s == "?" || s == "??" {
            Some(Self::Any)
        } else if let Ok(byte) = u8::from_str_radix(s, 16) {
            Some(Self::Byte(byte))
        } else {
            None
        }
    }

    pub fn to_byte(&self) -> Option<u8> {
        match self {
            Self::Any => None,
            Self::Byte(b) => Some(*b),
        }
    }
}

impl From<u8> for Wildcard {
    fn from(byte: u8) -> Self {
        Self::Byte(byte)
    }
}
