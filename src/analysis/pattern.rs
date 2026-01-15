// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::disassembler::DisassembledInstruction;
use std::collections::HashMap;

pub struct PatternRecognizer {
    patterns: Vec<InstructionPattern>,
}

impl PatternRecognizer {
    pub fn new() -> Self {
        let mut recognizer = Self {
            patterns: Vec::new(),
        };
        recognizer.load_default_patterns();
        recognizer
    }

    fn load_default_patterns(&mut self) {
        self.patterns.push(InstructionPattern {
            name: "VTableAccess".to_string(),
            description: "Virtual table method call".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("LDR").operand_contains("X0"),
                InstructionMatcher::mnemonic("LDR").operand_contains("[X").with_offset_range(0, 0x1000),
                InstructionMatcher::mnemonic("BLR"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "GlobalDataAccess".to_string(),
            description: "Access to global data via ADRP + ADD/LDR".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("ADRP"),
                InstructionMatcher::mnemonic_any(&["ADD", "LDR"]).operand_contains("[X"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "FunctionPrologue".to_string(),
            description: "Standard ARM64 function prologue".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("STP").operand_contains("X29").operand_contains("X30"),
                InstructionMatcher::mnemonic("MOV").operand_contains("X29").operand_contains("SP"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "TailCall".to_string(),
            description: "Tail call optimization pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("LDP").operand_contains("X29").operand_contains("X30"),
                InstructionMatcher::mnemonic("B").not_mnemonic("BL"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "SwitchTable".to_string(),
            description: "Switch statement via computed branch".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("CMP"),
                InstructionMatcher::mnemonic_any(&["B.HI", "B.LS", "B.GT", "B.LE"]),
                InstructionMatcher::mnemonic("ADR"),
                InstructionMatcher::mnemonic("LDRB").or(InstructionMatcher::mnemonic("LDRSW")),
                InstructionMatcher::mnemonic("ADD"),
                InstructionMatcher::mnemonic("BR"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "NullCheck".to_string(),
            description: "Null pointer check pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("CBZ").or(InstructionMatcher::mnemonic("CBNZ")),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "StringCompare".to_string(),
            description: "String comparison loop pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("LDRB"),
                InstructionMatcher::mnemonic("LDRB"),
                InstructionMatcher::mnemonic("CMP"),
                InstructionMatcher::mnemonic_any(&["B.NE", "B.EQ"]),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "MemoryCopy".to_string(),
            description: "Memory copy loop pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic_any(&["LDR", "LDP"]),
                InstructionMatcher::mnemonic_any(&["STR", "STP"]),
                InstructionMatcher::mnemonic("ADD").operand_contains("#"),
                InstructionMatcher::mnemonic("CMP").or(InstructionMatcher::mnemonic("SUBS")),
                InstructionMatcher::mnemonic_any(&["B.NE", "B.LT", "B.LE"]),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "RetainRelease".to_string(),
            description: "Objective-C retain/release pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("BL").operand_contains("retain")
                    .or(InstructionMatcher::mnemonic("BL").operand_contains("release")),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "LuaStackPush".to_string(),
            description: "Lua stack push operation pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("LDR").operand_contains("[X0"),
                InstructionMatcher::mnemonic("STR"),
                InstructionMatcher::mnemonic("ADD").operand_contains("#16").or(
                    InstructionMatcher::mnemonic("ADD").operand_contains("#0x10")
                ),
                InstructionMatcher::mnemonic("STR").operand_contains("[X0"),
            ],
        });

        self.patterns.push(InstructionPattern {
            name: "LuaStackPop".to_string(),
            description: "Lua stack pop operation pattern".to_string(),
            matchers: vec![
                InstructionMatcher::mnemonic("LDR").operand_contains("[X0"),
                InstructionMatcher::mnemonic("SUB").operand_contains("#16").or(
                    InstructionMatcher::mnemonic("SUB").operand_contains("#0x10")
                ),
                InstructionMatcher::mnemonic("STR").operand_contains("[X0"),
            ],
        });
    }

    pub fn find_patterns(&self, instructions: &[DisassembledInstruction]) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            let pattern_len = pattern.matchers.len();

            for i in 0..instructions.len().saturating_sub(pattern_len - 1) {
                if self.match_pattern(pattern, &instructions[i..]) {
                    matches.push(PatternMatch {
                        pattern_name: pattern.name.clone(),
                        start_address: instructions[i].address,
                        end_address: instructions[i + pattern_len - 1].address,
                        instructions: instructions[i..i + pattern_len].to_vec(),
                    });
                }
            }
        }

        matches
    }

    fn match_pattern(&self, pattern: &InstructionPattern, instructions: &[DisassembledInstruction]) -> bool {
        if instructions.len() < pattern.matchers.len() {
            return false;
        }

        for (idx, matcher) in pattern.matchers.iter().enumerate() {
            if !matcher.matches(&instructions[idx]) {
                return false;
            }
        }

        true
    }

    pub fn add_pattern(&mut self, pattern: InstructionPattern) {
        self.patterns.push(pattern);
    }

    pub fn patterns(&self) -> &[InstructionPattern] {
        &self.patterns
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InstructionPattern {
    pub name: String,
    pub description: String,
    pub matchers: Vec<InstructionMatcher>,
}

impl InstructionPattern {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matchers: Vec::new(),
        }
    }

    pub fn add_matcher(mut self, matcher: InstructionMatcher) -> Self {
        self.matchers.push(matcher);
        self
    }

    pub fn length(&self) -> usize {
        self.matchers.len()
    }
}

#[derive(Debug, Clone)]
pub struct InstructionMatcher {
    mnemonic_match: MnemonicMatch,
    operand_contains: Vec<String>,
    operand_not_contains: Vec<String>,
    offset_range: Option<(i64, i64)>,
    alternative: Option<Box<InstructionMatcher>>,
}

#[derive(Debug, Clone)]
enum MnemonicMatch {
    Exact(String),
    Any(Vec<String>),
    Not(String),
    Any_,
}

impl InstructionMatcher {
    pub fn mnemonic(mnemonic: &str) -> Self {
        Self {
            mnemonic_match: MnemonicMatch::Exact(mnemonic.to_string()),
            operand_contains: Vec::new(),
            operand_not_contains: Vec::new(),
            offset_range: None,
            alternative: None,
        }
    }

    pub fn mnemonic_any(mnemonics: &[&str]) -> Self {
        Self {
            mnemonic_match: MnemonicMatch::Any(mnemonics.iter().map(|s| s.to_string()).collect()),
            operand_contains: Vec::new(),
            operand_not_contains: Vec::new(),
            offset_range: None,
            alternative: None,
        }
    }

    pub fn any() -> Self {
        Self {
            mnemonic_match: MnemonicMatch::Any_,
            operand_contains: Vec::new(),
            operand_not_contains: Vec::new(),
            offset_range: None,
            alternative: None,
        }
    }

    pub fn not_mnemonic(mut self, mnemonic: &str) -> Self {
        self.operand_not_contains.push(mnemonic.to_string());
        self
    }

    pub fn operand_contains(mut self, substring: &str) -> Self {
        self.operand_contains.push(substring.to_string());
        self
    }

    pub fn operand_not_contains(mut self, substring: &str) -> Self {
        self.operand_not_contains.push(substring.to_string());
        self
    }

    pub fn with_offset_range(mut self, min: i64, max: i64) -> Self {
        self.offset_range = Some((min, max));
        self
    }

    pub fn or(mut self, other: InstructionMatcher) -> Self {
        self.alternative = Some(Box::new(other));
        self
    }

    pub fn matches(&self, instruction: &DisassembledInstruction) -> bool {
        if self.matches_internal(instruction) {
            return true;
        }

        if let Some(ref alt) = self.alternative {
            return alt.matches(instruction);
        }

        false
    }

    fn matches_internal(&self, instruction: &DisassembledInstruction) -> bool {
        let mnemonic_matches = match &self.mnemonic_match {
            MnemonicMatch::Exact(m) => instruction.mnemonic == *m,
            MnemonicMatch::Any(ms) => ms.iter().any(|m| instruction.mnemonic == *m),
            MnemonicMatch::Not(m) => instruction.mnemonic != *m,
            MnemonicMatch::Any_ => true,
        };

        if !mnemonic_matches {
            return false;
        }

        for substr in &self.operand_contains {
            if !instruction.op_str.contains(substr) {
                return false;
            }
        }

        for substr in &self.operand_not_contains {
            if instruction.op_str.contains(substr) {
                return false;
            }
        }

        if let Some((min, max)) = self.offset_range {
            if let Some(offset) = self.extract_offset(&instruction.op_str) {
                if offset < min || offset > max {
                    return false;
                }
            }
        }

        true
    }

    fn extract_offset(&self, op_str: &str) -> Option<i64> {
        for part in op_str.split(|c: char| c == ',' || c == ' ' || c == '[' || c == ']') {
            let trimmed = part.trim().trim_start_matches('#');
            if let Ok(val) = trimmed.parse::<i64>() {
                return Some(val);
            }
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                if let Ok(val) = i64::from_str_radix(&trimmed[2..], 16) {
                    return Some(val);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub start_address: Address,
    pub end_address: Address,
    pub instructions: Vec<DisassembledInstruction>,
}

impl PatternMatch {
    pub fn length(&self) -> usize {
        self.instructions.len()
    }

    pub fn size_bytes(&self) -> u64 {
        self.end_address.as_u64() - self.start_address.as_u64() +
            self.instructions.last().map(|i| i.size as u64).unwrap_or(4)
    }
}

pub struct PatternStatistics {
    pub pattern_counts: HashMap<String, usize>,
    pub total_matches: usize,
}

impl PatternStatistics {
    pub fn from_matches(matches: &[PatternMatch]) -> Self {
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();

        for m in matches {
            *pattern_counts.entry(m.pattern_name.clone()).or_insert(0) += 1;
        }

        Self {
            pattern_counts,
            total_matches: matches.len(),
        }
    }

    pub fn most_common(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut counts: Vec<_> = self.pattern_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));
        counts.into_iter().take(n).collect()
    }
}
