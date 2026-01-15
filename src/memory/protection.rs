// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protection {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    ReadWrite = 3,
    ReadExecute = 5,
    ReadWriteExecute = 7,
}

impl Protection {
    pub fn from_flags(flags: u32) -> Self {
        match flags & 7 {
            0 => Self::None,
            1 => Self::Read,
            2 => Self::Write,
            3 => Self::ReadWrite,
            4 => Self::Execute,
            5 => Self::ReadExecute,
            7 => Self::ReadWriteExecute,
            _ => Self::None,
        }
    }

    pub fn to_flags(self) -> u32 {
        self as u32
    }

    pub fn can_read(self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite | Self::ReadExecute | Self::ReadWriteExecute)
    }

    pub fn can_write(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite | Self::ReadWriteExecute)
    }

    pub fn can_execute(self) -> bool {
        matches!(self, Self::Execute | Self::ReadExecute | Self::ReadWriteExecute)
    }

    pub fn is_readable(self) -> bool {
        self.can_read()
    }

    pub fn is_writable(self) -> bool {
        self.can_write()
    }

    pub fn is_executable(self) -> bool {
        self.can_execute()
    }
}

impl fmt::Display for Protection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "---"),
            Self::Read => write!(f, "r--"),
            Self::Write => write!(f, "-w-"),
            Self::Execute => write!(f, "--x"),
            Self::ReadWrite => write!(f, "rw-"),
            Self::ReadExecute => write!(f, "r-x"),
            Self::ReadWriteExecute => write!(f, "rwx"),
        }
    }
}
