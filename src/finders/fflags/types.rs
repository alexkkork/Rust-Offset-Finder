// Wed Jan 15 2026 - Alex

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFlag {
    pub name: String,
    pub flag_type: FFlagType,
    pub value: FFlagValue,
    pub address: u64,
    pub default_value: Option<FFlagValue>,
}

impl FFlag {
    pub fn new(name: String, flag_type: FFlagType, value: FFlagValue, address: u64) -> Self {
        Self {
            name,
            flag_type,
            value,
            address,
            default_value: None,
        }
    }

    pub fn with_default(mut self, default: FFlagValue) -> Self {
        self.default_value = Some(default);
        self
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self.flag_type, 
            FFlagType::DFFlag | FFlagType::DFInt | FFlagType::DFString | FFlagType::DFLog
        )
    }

    pub fn is_fast(&self) -> bool {
        matches!(self.flag_type,
            FFlagType::FFlag | FFlagType::FInt | FFlagType::FString | FFlagType::FLog
        )
    }

    pub fn is_synchronized(&self) -> bool {
        matches!(self.flag_type,
            FFlagType::SFFlag | FFlagType::SFInt | FFlagType::SFString | FFlagType::SFLog
        )
    }

    pub fn prefix(&self) -> &'static str {
        match self.flag_type {
            FFlagType::FFlag => "FFlag",
            FFlagType::FInt => "FInt",
            FFlagType::FString => "FString",
            FFlagType::FLog => "FLog",
            FFlagType::DFFlag => "DFFlag",
            FFlagType::DFInt => "DFInt",
            FFlagType::DFString => "DFString",
            FFlagType::DFLog => "DFLog",
            FFlagType::SFFlag => "SFFlag",
            FFlagType::SFInt => "SFInt",
            FFlagType::SFString => "SFString",
            FFlagType::SFLog => "SFLog",
            FFlagType::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for FFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{} = {} @ 0x{:X}", self.prefix(), self.name, self.value, self.address)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FFlagType {
    FFlag,
    FInt,
    FString,
    FLog,
    DFFlag,
    DFInt,
    DFString,
    DFLog,
    SFFlag,
    SFInt,
    SFString,
    SFLog,
    Unknown,
}

impl FFlagType {
    pub fn from_prefix(prefix: &str) -> Self {
        match prefix {
            "FFlag" => FFlagType::FFlag,
            "FInt" => FFlagType::FInt,
            "FString" => FFlagType::FString,
            "FLog" => FFlagType::FLog,
            "DFFlag" => FFlagType::DFFlag,
            "DFInt" => FFlagType::DFInt,
            "DFString" => FFlagType::DFString,
            "DFLog" => FFlagType::DFLog,
            "SFFlag" => FFlagType::SFFlag,
            "SFInt" => FFlagType::SFInt,
            "SFString" => FFlagType::SFString,
            "SFLog" => FFlagType::SFLog,
            _ => FFlagType::Unknown,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, FFlagType::FFlag | FFlagType::DFFlag | FFlagType::SFFlag)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, FFlagType::FInt | FFlagType::DFInt | FFlagType::SFInt)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, FFlagType::FString | FFlagType::DFString | FFlagType::SFString)
    }

    pub fn is_log(&self) -> bool {
        matches!(self, FFlagType::FLog | FFlagType::DFLog | FFlagType::SFLog)
    }
}

impl fmt::Display for FFlagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FFlagType::FFlag => "FFlag",
            FFlagType::FInt => "FInt",
            FFlagType::FString => "FString",
            FFlagType::FLog => "FLog",
            FFlagType::DFFlag => "DFFlag",
            FFlagType::DFInt => "DFInt",
            FFlagType::DFString => "DFString",
            FFlagType::DFLog => "DFLog",
            FFlagType::SFFlag => "SFFlag",
            FFlagType::SFInt => "SFInt",
            FFlagType::SFString => "SFString",
            FFlagType::SFLog => "SFLog",
            FFlagType::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FFlagValue {
    Bool(bool),
    Int(i64),
    String(String),
    Log(i32),
    Unknown,
}

impl FFlagValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FFlagValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            FFlagValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            FFlagValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_log(&self) -> Option<i32> {
        match self {
            FFlagValue::Log(l) => Some(*l),
            _ => None,
        }
    }
}

impl fmt::Display for FFlagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FFlagValue::Bool(b) => write!(f, "{}", b),
            FFlagValue::Int(i) => write!(f, "{}", i),
            FFlagValue::String(s) => write!(f, "\"{}\"", s),
            FFlagValue::Log(l) => write!(f, "Log({})", l),
            FFlagValue::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFlagCategory {
    pub name: String,
    pub flags: Vec<FFlag>,
}

impl FFlagCategory {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            flags: Vec::new(),
        }
    }

    pub fn add(&mut self, flag: FFlag) {
        self.flags.push(flag);
    }

    pub fn count(&self) -> usize {
        self.flags.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFlagCollection {
    pub flags: Vec<FFlag>,
    pub categories: Vec<FFlagCategory>,
    pub total_count: usize,
    pub by_type: FFlagStats,
}

impl FFlagCollection {
    pub fn new() -> Self {
        Self {
            flags: Vec::new(),
            categories: Vec::new(),
            total_count: 0,
            by_type: FFlagStats::default(),
        }
    }

    pub fn add(&mut self, flag: FFlag) {
        self.by_type.increment(&flag.flag_type);
        self.flags.push(flag);
        self.total_count += 1;
    }

    pub fn get(&self, name: &str) -> Option<&FFlag> {
        self.flags.iter().find(|f| f.name == name)
    }

    pub fn filter_by_type(&self, flag_type: FFlagType) -> Vec<&FFlag> {
        self.flags.iter().filter(|f| f.flag_type == flag_type).collect()
    }

    pub fn filter_by_prefix(&self, prefix: &str) -> Vec<&FFlag> {
        self.flags.iter().filter(|f| f.name.starts_with(prefix)).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&FFlag> {
        let query_lower = query.to_lowercase();
        self.flags.iter()
            .filter(|f| f.name.to_lowercase().contains(&query_lower))
            .collect()
    }
}

impl Default for FFlagCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FFlagStats {
    pub fflags: usize,
    pub fints: usize,
    pub fstrings: usize,
    pub flogs: usize,
    pub dfflags: usize,
    pub dfints: usize,
    pub dfstrings: usize,
    pub dflogs: usize,
    pub sfflags: usize,
    pub sfints: usize,
    pub sfstrings: usize,
    pub sflogs: usize,
    pub unknown: usize,
}

impl FFlagStats {
    pub fn increment(&mut self, flag_type: &FFlagType) {
        match flag_type {
            FFlagType::FFlag => self.fflags += 1,
            FFlagType::FInt => self.fints += 1,
            FFlagType::FString => self.fstrings += 1,
            FFlagType::FLog => self.flogs += 1,
            FFlagType::DFFlag => self.dfflags += 1,
            FFlagType::DFInt => self.dfints += 1,
            FFlagType::DFString => self.dfstrings += 1,
            FFlagType::DFLog => self.dflogs += 1,
            FFlagType::SFFlag => self.sfflags += 1,
            FFlagType::SFInt => self.sfints += 1,
            FFlagType::SFString => self.sfstrings += 1,
            FFlagType::SFLog => self.sflogs += 1,
            FFlagType::Unknown => self.unknown += 1,
        }
    }

    pub fn total(&self) -> usize {
        self.fflags + self.fints + self.fstrings + self.flogs +
        self.dfflags + self.dfints + self.dfstrings + self.dflogs +
        self.sfflags + self.sfints + self.sfstrings + self.sflogs +
        self.unknown
    }

    pub fn fast_flags_total(&self) -> usize {
        self.fflags + self.fints + self.fstrings + self.flogs
    }

    pub fn dynamic_flags_total(&self) -> usize {
        self.dfflags + self.dfints + self.dfstrings + self.dflogs
    }

    pub fn sync_flags_total(&self) -> usize {
        self.sfflags + self.sfints + self.sfstrings + self.sflogs
    }
}
