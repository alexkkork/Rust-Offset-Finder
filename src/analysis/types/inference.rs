// Wed Jan 15 2026 - Alex

use super::{DataType, PrimitiveType, PointerType, PointerTarget, PointerAnalyzer};
use crate::memory::{Address, MemoryReader};
use std::collections::HashMap;
use std::sync::Arc;

pub struct TypeInferenceEngine {
    reader: Arc<dyn MemoryReader>,
    cache: HashMap<u64, InferredType>,
    text_range: (u64, u64),
    data_range: (u64, u64),
}

#[derive(Debug, Clone)]
pub struct InferredType {
    pub data_type: DataType,
    pub confidence: f64,
    pub evidence: Vec<TypeEvidence>,
}

#[derive(Debug, Clone)]
pub enum TypeEvidence {
    PatternMatch(String),
    ValueRange(u64, u64),
    Alignment(usize),
    PointerTarget(PointerTarget),
    StringContent,
    FloatPattern,
    NullTerminated,
    VTablePointer,
    FunctionPointer,
}

impl TypeInferenceEngine {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            cache: HashMap::new(),
            text_range: (0, 0),
            data_range: (0, 0),
        }
    }

    pub fn set_text_range(&mut self, base: u64, size: u64) {
        self.text_range = (base, base + size);
    }

    pub fn set_data_range(&mut self, base: u64, size: u64) {
        self.data_range = (base, base + size);
    }

    pub fn infer_at(&mut self, addr: Address) -> InferredType {
        if let Some(cached) = self.cache.get(&addr.as_u64()) {
            return cached.clone();
        }

        let result = self.do_infer(addr);
        self.cache.insert(addr.as_u64(), result.clone());
        result
    }

    fn do_infer(&self, addr: Address) -> InferredType {
        let bytes = match self.reader.read_bytes(addr, 8) {
            Ok(b) => b,
            Err(_) => return InferredType::unknown(),
        };

        let value_u64 = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let value_f64 = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        let mut evidence = Vec::new();
        let mut candidates: Vec<(DataType, f64)> = Vec::new();

        if self.looks_like_pointer(value_u64) {
            let target = PointerAnalyzer::likely_pointer_target(
                value_u64,
                self.text_range.0,
                self.text_range.1 - self.text_range.0,
                self.data_range.0,
                self.data_range.1 - self.data_range.0,
            );

            evidence.push(TypeEvidence::PointerTarget(target));

            match target {
                PointerTarget::Code => {
                    evidence.push(TypeEvidence::FunctionPointer);
                    candidates.push((PointerType::to_void(), 0.85));
                }
                PointerTarget::Data => {
                    candidates.push((PointerType::to_void(), 0.80));
                }
                PointerTarget::Heap => {
                    candidates.push((PointerType::to_void(), 0.60));
                }
                _ => {}
            }
        }

        if self.looks_like_float(value_f64) {
            evidence.push(TypeEvidence::FloatPattern);
            candidates.push((DataType::f64(), 0.70));
        }

        if self.looks_like_small_integer(value_u64) {
            let value_i64 = value_u64 as i64;

            if value_u64 <= 0xFF {
                evidence.push(TypeEvidence::ValueRange(0, 0xFF));
                candidates.push((DataType::u8(), 0.50));
            } else if value_u64 <= 0xFFFF {
                evidence.push(TypeEvidence::ValueRange(0, 0xFFFF));
                candidates.push((DataType::u16(), 0.50));
            } else if value_u64 <= 0xFFFFFFFF {
                evidence.push(TypeEvidence::ValueRange(0, 0xFFFFFFFF));
                candidates.push((DataType::u32(), 0.55));
            } else {
                candidates.push((DataType::u64(), 0.45));
            }
        }

        if self.looks_like_string_pointer(value_u64) {
            evidence.push(TypeEvidence::StringContent);
            candidates.push((PointerType::to_i8(), 0.75));
        }

        if candidates.is_empty() {
            candidates.push((DataType::u64(), 0.30));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        InferredType {
            data_type: candidates[0].0.clone(),
            confidence: candidates[0].1,
            evidence,
        }
    }

    fn looks_like_pointer(&self, value: u64) -> bool {
        if value == 0 {
            return false;
        }

        if value < 0x1000 {
            return false;
        }

        let top_bits = value >> 48;
        if top_bits != 0 && top_bits != 0xFFFF {
            return false;
        }

        if value & 0x3 != 0 {
            return false;
        }

        true
    }

    fn looks_like_float(&self, value: f64) -> bool {
        if value.is_nan() || value.is_infinite() {
            return false;
        }

        let abs = value.abs();
        abs > 1e-10 && abs < 1e10
    }

    fn looks_like_small_integer(&self, value: u64) -> bool {
        value < 0x100000000
    }

    fn looks_like_string_pointer(&self, value: u64) -> bool {
        if !self.looks_like_pointer(value) {
            return false;
        }

        if let Ok(bytes) = self.reader.read_bytes(Address::new(value), 16) {
            let printable_count = bytes.iter()
                .take_while(|&&b| b != 0)
                .filter(|&&b| b >= 0x20 && b < 0x7F)
                .count();

            printable_count >= 4
        } else {
            false
        }
    }

    pub fn infer_array(&mut self, addr: Address, count: usize) -> Vec<InferredType> {
        let mut results = Vec::with_capacity(count);
        let mut current = addr;

        for _ in 0..count {
            let inferred = self.infer_at(current);
            let size = inferred.data_type.size().max(1);
            results.push(inferred);
            current = current + size as u64;
        }

        results
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl InferredType {
    pub fn unknown() -> Self {
        Self {
            data_type: DataType::Unknown,
            confidence: 0.0,
            evidence: Vec::new(),
        }
    }

    pub fn with_type(data_type: DataType, confidence: f64) -> Self {
        Self {
            data_type,
            confidence,
            evidence: Vec::new(),
        }
    }

    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.7
    }

    pub fn is_pointer(&self) -> bool {
        self.data_type.is_pointer()
    }
}
