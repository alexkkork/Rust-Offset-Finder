// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryRegion};
use crate::pattern::{Pattern, PatternScanner};
use std::collections::HashMap;
use std::fmt;

/// A YARA-style rule for pattern matching
#[derive(Debug, Clone)]
pub struct YaraRule {
    /// Rule name
    pub name: String,
    /// Rule metadata
    pub meta: HashMap<String, MetaValue>,
    /// String patterns
    pub strings: Vec<YaraString>,
    /// Condition for the rule
    pub condition: Condition,
    /// Tags
    pub tags: Vec<String>,
    /// Whether the rule is private
    pub private: bool,
    /// Whether the rule is global
    pub global: bool,
}

impl YaraRule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            meta: HashMap::new(),
            strings: Vec::new(),
            condition: Condition::Any,
            tags: Vec::new(),
            private: false,
            global: false,
        }
    }

    pub fn with_meta(mut self, key: &str, value: MetaValue) -> Self {
        self.meta.insert(key.to_string(), value);
        self
    }

    pub fn with_string(mut self, string: YaraString) -> Self {
        self.strings.push(string);
        self
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn private(mut self) -> Self {
        self.private = true;
        self
    }

    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Convert to YARA syntax string
    pub fn to_yara(&self) -> String {
        let mut yara = String::new();

        // Rule declaration
        if self.private {
            yara.push_str("private ");
        }
        if self.global {
            yara.push_str("global ");
        }
        yara.push_str(&format!("rule {} ", self.name));
        
        if !self.tags.is_empty() {
            yara.push_str(": ");
            yara.push_str(&self.tags.join(" "));
            yara.push(' ');
        }
        
        yara.push_str("{\n");

        // Meta section
        if !self.meta.is_empty() {
            yara.push_str("  meta:\n");
            for (key, value) in &self.meta {
                yara.push_str(&format!("    {} = {}\n", key, value));
            }
        }

        // Strings section
        if !self.strings.is_empty() {
            yara.push_str("  strings:\n");
            for string in &self.strings {
                yara.push_str(&format!("    {}\n", string.to_yara()));
            }
        }

        // Condition section
        yara.push_str("  condition:\n");
        yara.push_str(&format!("    {}\n", self.condition.to_yara()));

        yara.push_str("}\n");
        yara
    }
}

impl fmt::Display for YaraRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_yara())
    }
}

/// Metadata value types
#[derive(Debug, Clone)]
pub enum MetaValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

impl fmt::Display for MetaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaValue::String(s) => write!(f, "\"{}\"", s),
            MetaValue::Integer(i) => write!(f, "{}", i),
            MetaValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}

/// A string pattern in YARA format
#[derive(Debug, Clone)]
pub struct YaraString {
    /// Identifier (without $)
    pub identifier: String,
    /// Pattern type
    pub pattern: YaraPattern,
    /// Modifiers
    pub modifiers: StringModifiers,
}

impl YaraString {
    pub fn text(id: &str, text: &str) -> Self {
        Self {
            identifier: id.to_string(),
            pattern: YaraPattern::Text(text.to_string()),
            modifiers: StringModifiers::default(),
        }
    }

    pub fn hex(id: &str, hex: &str) -> Self {
        Self {
            identifier: id.to_string(),
            pattern: YaraPattern::Hex(hex.to_string()),
            modifiers: StringModifiers::default(),
        }
    }

    pub fn regex(id: &str, regex: &str) -> Self {
        Self {
            identifier: id.to_string(),
            pattern: YaraPattern::Regex(regex.to_string()),
            modifiers: StringModifiers::default(),
        }
    }

    pub fn with_modifiers(mut self, modifiers: StringModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn nocase(mut self) -> Self {
        self.modifiers.nocase = true;
        self
    }

    pub fn wide(mut self) -> Self {
        self.modifiers.wide = true;
        self
    }

    pub fn ascii(mut self) -> Self {
        self.modifiers.ascii = true;
        self
    }

    pub fn fullword(mut self) -> Self {
        self.modifiers.fullword = true;
        self
    }

    pub fn xor(mut self) -> Self {
        self.modifiers.xor = true;
        self
    }

    pub fn to_yara(&self) -> String {
        let mut yara = format!("${} = ", self.identifier);
        
        match &self.pattern {
            YaraPattern::Text(s) => yara.push_str(&format!("\"{}\"", s)),
            YaraPattern::Hex(h) => yara.push_str(&format!("{{ {} }}", h)),
            YaraPattern::Regex(r) => yara.push_str(&format!("/{}/", r)),
        }

        let mods = self.modifiers.to_yara();
        if !mods.is_empty() {
            yara.push(' ');
            yara.push_str(&mods);
        }

        yara
    }

    /// Convert to a Pattern for matching
    pub fn to_pattern(&self) -> Option<Pattern> {
        match &self.pattern {
            YaraPattern::Hex(hex) => parse_yara_hex(hex),
            YaraPattern::Text(text) => Some(Pattern::from_bytes(text.as_bytes())),
            YaraPattern::Regex(_) => None, // Regex needs special handling
        }
    }
}

/// Pattern type in YARA
#[derive(Debug, Clone)]
pub enum YaraPattern {
    Text(String),
    Hex(String),
    Regex(String),
}

/// String modifiers
#[derive(Debug, Clone, Default)]
pub struct StringModifiers {
    pub nocase: bool,
    pub wide: bool,
    pub ascii: bool,
    pub fullword: bool,
    pub xor: bool,
    pub xor_range: Option<(u8, u8)>,
    pub base64: bool,
    pub base64wide: bool,
}

impl StringModifiers {
    pub fn to_yara(&self) -> String {
        let mut mods = Vec::new();
        if self.nocase { mods.push("nocase"); }
        if self.wide { mods.push("wide"); }
        if self.ascii { mods.push("ascii"); }
        if self.fullword { mods.push("fullword"); }
        if self.xor { mods.push("xor"); }
        if self.base64 { mods.push("base64"); }
        if self.base64wide { mods.push("base64wide"); }
        mods.join(" ")
    }
}

/// Condition for rule matching
#[derive(Debug, Clone)]
pub enum Condition {
    /// Any string matches
    Any,
    /// All strings match
    All,
    /// Specific number of strings match
    Count(usize),
    /// At least N strings match
    AtLeast(usize),
    /// Specific strings must match
    Strings(Vec<String>),
    /// Boolean expression
    Expression(Box<ConditionExpr>),
    /// Custom condition string
    Custom(String),
}

impl Condition {
    pub fn to_yara(&self) -> String {
        match self {
            Condition::Any => "any of them".to_string(),
            Condition::All => "all of them".to_string(),
            Condition::Count(n) => format!("{} of them", n),
            Condition::AtLeast(n) => format!("{} of them", n),
            Condition::Strings(ids) => {
                let refs: Vec<String> = ids.iter().map(|id| format!("${}", id)).collect();
                refs.join(" and ")
            }
            Condition::Expression(expr) => expr.to_yara(),
            Condition::Custom(s) => s.clone(),
        }
    }
}

/// Boolean expression for conditions
#[derive(Debug, Clone)]
pub enum ConditionExpr {
    StringRef(String),
    StringAt(String, u64),
    StringIn(String, u64, u64),
    Not(Box<ConditionExpr>),
    And(Box<ConditionExpr>, Box<ConditionExpr>),
    Or(Box<ConditionExpr>, Box<ConditionExpr>),
    Count(String, usize),
    FileSize(Comparison, u64),
    Entrypoint,
}

impl ConditionExpr {
    pub fn to_yara(&self) -> String {
        match self {
            ConditionExpr::StringRef(id) => format!("${}", id),
            ConditionExpr::StringAt(id, offset) => format!("${} at {}", id, offset),
            ConditionExpr::StringIn(id, start, end) => format!("${} in ({}..{})", id, start, end),
            ConditionExpr::Not(expr) => format!("not ({})", expr.to_yara()),
            ConditionExpr::And(a, b) => format!("({}) and ({})", a.to_yara(), b.to_yara()),
            ConditionExpr::Or(a, b) => format!("({}) or ({})", a.to_yara(), b.to_yara()),
            ConditionExpr::Count(id, n) => format!("#{} == {}", id, n),
            ConditionExpr::FileSize(cmp, size) => format!("filesize {} {}", cmp.to_yara(), size),
            ConditionExpr::Entrypoint => "entrypoint".to_string(),
        }
    }
}

/// Comparison operators
#[derive(Debug, Clone, Copy)]
pub enum Comparison {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

impl Comparison {
    pub fn to_yara(&self) -> &'static str {
        match self {
            Comparison::Equal => "==",
            Comparison::NotEqual => "!=",
            Comparison::LessThan => "<",
            Comparison::LessEqual => "<=",
            Comparison::GreaterThan => ">",
            Comparison::GreaterEqual => ">=",
        }
    }
}

/// Parse YARA hex pattern string to Pattern
fn parse_yara_hex(hex: &str) -> Option<Pattern> {
    let mut bytes = Vec::new();
    let mut mask = Vec::new();
    
    let cleaned: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    let mut chars = cleaned.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '?' => {
                // Check for ?? (full wildcard) or ?X (half wildcard)
                if chars.peek() == Some(&'?') {
                    chars.next();
                    bytes.push(0);
                    mask.push(false);
                } else {
                    // Half wildcard not fully supported, treat as full
                    bytes.push(0);
                    mask.push(false);
                }
            }
            '[' => {
                // Jump pattern [min-max]
                let mut range = String::new();
                while let Some(&rc) = chars.peek() {
                    chars.next();
                    if rc == ']' {
                        break;
                    }
                    range.push(rc);
                }
                // For now, treat jumps as wildcards (simplified)
                let count: usize = range.split('-').next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                for _ in 0..count {
                    bytes.push(0);
                    mask.push(false);
                }
            }
            '(' => {
                // Alternative patterns - simplified, take first
                let mut alt = String::new();
                let mut depth = 1;
                while let Some(&ac) = chars.peek() {
                    chars.next();
                    if ac == '(' {
                        depth += 1;
                    } else if ac == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if ac == '|' && depth == 1 {
                        break; // Take first alternative
                    }
                    alt.push(ac);
                }
                if let Some(p) = parse_yara_hex(&alt) {
                    bytes.extend(p.bytes());
                    mask.extend(p.mask().iter().copied());
                }
            }
            _ if c.is_ascii_hexdigit() => {
                if let Some(&c2) = chars.peek() {
                    if c2.is_ascii_hexdigit() {
                        chars.next();
                        let byte = u8::from_str_radix(&format!("{}{}", c, c2), 16).ok()?;
                        bytes.push(byte);
                        mask.push(true);
                    }
                }
            }
            _ => {}
        }
    }

    if bytes.is_empty() {
        return None;
    }

    Some(Pattern::new(bytes, mask))
}

/// YARA rule matcher
pub struct YaraMatcher {
    rules: Vec<YaraRule>,
}

impl YaraMatcher {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: YaraRule) {
        self.rules.push(rule);
    }

    pub fn add_rules(&mut self, rules: Vec<YaraRule>) {
        self.rules.extend(rules);
    }

    /// Match all rules against a memory region
    pub fn match_rules(&self, reader: &dyn MemoryReader, region: &MemoryRegion) -> Vec<YaraMatch> {
        let mut matches = Vec::new();

        for rule in &self.rules {
            if let Some(m) = self.match_rule(reader, region, rule) {
                matches.push(m);
            }
        }

        matches
    }

    fn match_rule(&self, reader: &dyn MemoryReader, region: &MemoryRegion, rule: &YaraRule) -> Option<YaraMatch> {
        let mut string_matches: HashMap<String, Vec<Address>> = HashMap::new();
        let scanner = PatternScanner::new();

        // Find all string matches
        for string in &rule.strings {
            if let Some(pattern) = string.to_pattern() {
                let matches = scanner.scan(reader, &pattern, &[region.clone()]);
                if !matches.is_empty() {
                    string_matches.insert(string.identifier.clone(), matches);
                }
            }
        }

        // Evaluate condition
        let matched = self.evaluate_condition(&rule.condition, &string_matches);

        if matched {
            Some(YaraMatch {
                rule_name: rule.name.clone(),
                tags: rule.tags.clone(),
                string_matches,
            })
        } else {
            None
        }
    }

    fn evaluate_condition(&self, condition: &Condition, matches: &HashMap<String, Vec<Address>>) -> bool {
        match condition {
            Condition::Any => !matches.is_empty(),
            Condition::All => matches.len() == matches.len(), // All strings found
            Condition::Count(n) => matches.len() >= *n,
            Condition::AtLeast(n) => matches.len() >= *n,
            Condition::Strings(ids) => {
                ids.iter().all(|id| matches.contains_key(id))
            }
            Condition::Expression(expr) => self.evaluate_expression(expr, matches),
            Condition::Custom(_) => true, // Custom conditions need parsing
        }
    }

    fn evaluate_expression(&self, expr: &ConditionExpr, matches: &HashMap<String, Vec<Address>>) -> bool {
        match expr {
            ConditionExpr::StringRef(id) => matches.contains_key(id),
            ConditionExpr::Not(inner) => !self.evaluate_expression(inner, matches),
            ConditionExpr::And(a, b) => {
                self.evaluate_expression(a, matches) && self.evaluate_expression(b, matches)
            }
            ConditionExpr::Or(a, b) => {
                self.evaluate_expression(a, matches) || self.evaluate_expression(b, matches)
            }
            ConditionExpr::Count(id, n) => {
                matches.get(id).map(|m| m.len() >= *n).unwrap_or(false)
            }
            _ => true,
        }
    }

    pub fn rules(&self) -> &[YaraRule] {
        &self.rules
    }
}

impl Default for YaraMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a YARA rule match
#[derive(Debug, Clone)]
pub struct YaraMatch {
    pub rule_name: String,
    pub tags: Vec<String>,
    pub string_matches: HashMap<String, Vec<Address>>,
}

impl YaraMatch {
    pub fn total_matches(&self) -> usize {
        self.string_matches.values().map(|v| v.len()).sum()
    }

    pub fn matched_strings(&self) -> Vec<&String> {
        self.string_matches.keys().collect()
    }
}

impl fmt::Display for YaraMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule: {}", self.rule_name)?;
        if !self.tags.is_empty() {
            write!(f, " [{}]", self.tags.join(", "))?;
        }
        writeln!(f)?;
        for (id, addrs) in &self.string_matches {
            write!(f, "  ${}: {} match(es)", id, addrs.len())?;
            if addrs.len() <= 3 {
                let addr_strs: Vec<String> = addrs.iter()
                    .map(|a| format!("0x{:x}", a.as_u64()))
                    .collect();
                write!(f, " at {}", addr_strs.join(", "))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Builder for YARA rules
pub struct YaraRuleBuilder {
    rule: YaraRule,
}

impl YaraRuleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            rule: YaraRule::new(name),
        }
    }

    pub fn meta_string(mut self, key: &str, value: &str) -> Self {
        self.rule.meta.insert(key.to_string(), MetaValue::String(value.to_string()));
        self
    }

    pub fn meta_int(mut self, key: &str, value: i64) -> Self {
        self.rule.meta.insert(key.to_string(), MetaValue::Integer(value));
        self
    }

    pub fn meta_bool(mut self, key: &str, value: bool) -> Self {
        self.rule.meta.insert(key.to_string(), MetaValue::Boolean(value));
        self
    }

    pub fn text_string(mut self, id: &str, text: &str) -> Self {
        self.rule.strings.push(YaraString::text(id, text));
        self
    }

    pub fn hex_string(mut self, id: &str, hex: &str) -> Self {
        self.rule.strings.push(YaraString::hex(id, hex));
        self
    }

    pub fn regex_string(mut self, id: &str, regex: &str) -> Self {
        self.rule.strings.push(YaraString::regex(id, regex));
        self
    }

    pub fn condition_any(mut self) -> Self {
        self.rule.condition = Condition::Any;
        self
    }

    pub fn condition_all(mut self) -> Self {
        self.rule.condition = Condition::All;
        self
    }

    pub fn condition_count(mut self, n: usize) -> Self {
        self.rule.condition = Condition::Count(n);
        self
    }

    pub fn condition_strings(mut self, ids: &[&str]) -> Self {
        self.rule.condition = Condition::Strings(ids.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.rule.tags.push(tag.to_string());
        self
    }

    pub fn private(mut self) -> Self {
        self.rule.private = true;
        self
    }

    pub fn global(mut self) -> Self {
        self.rule.global = true;
        self
    }

    pub fn build(self) -> YaraRule {
        self.rule
    }
}

/// Parse a YARA rule from string (simplified parser)
pub fn parse_yara_rule(source: &str) -> Option<YaraRule> {
    let source = source.trim();
    
    // Find rule name
    let rule_start = source.find("rule ")?;
    let after_rule = &source[rule_start + 5..];
    let name_end = after_rule.find(|c: char| c == '{' || c == ':' || c.is_whitespace())?;
    let name = after_rule[..name_end].trim();

    let mut rule = YaraRule::new(name);

    // Parse strings section
    if let Some(strings_start) = source.find("strings:") {
        let strings_section = &source[strings_start..];
        if let Some(cond_start) = strings_section.find("condition:") {
            let strings_content = &strings_section[8..cond_start].trim();
            for line in strings_content.lines() {
                if let Some(string) = parse_string_line(line.trim()) {
                    rule.strings.push(string);
                }
            }
        }
    }

    // Parse condition (simplified)
    if let Some(cond_start) = source.find("condition:") {
        let cond_section = &source[cond_start + 10..];
        if let Some(end) = cond_section.find('}') {
            let cond_text = cond_section[..end].trim();
            rule.condition = parse_condition(cond_text);
        }
    }

    Some(rule)
}

fn parse_string_line(line: &str) -> Option<YaraString> {
    if !line.starts_with('$') {
        return None;
    }

    let eq_pos = line.find('=')?;
    let id = line[1..eq_pos].trim();
    let value_part = line[eq_pos + 1..].trim();

    if value_part.starts_with('"') && value_part.len() > 2 {
        // Text string
        let end_quote = value_part[1..].find('"')?;
        let text = &value_part[1..=end_quote];
        Some(YaraString::text(id, text))
    } else if value_part.starts_with('{') {
        // Hex string
        let end_brace = value_part.find('}')?;
        let hex = &value_part[1..end_brace].trim();
        Some(YaraString::hex(id, hex))
    } else if value_part.starts_with('/') {
        // Regex
        let end_slash = value_part[1..].find('/')?;
        let regex = &value_part[1..=end_slash];
        Some(YaraString::regex(id, regex))
    } else {
        None
    }
}

fn parse_condition(text: &str) -> Condition {
    let text = text.trim().to_lowercase();
    
    if text == "any of them" {
        Condition::Any
    } else if text == "all of them" {
        Condition::All
    } else if text.ends_with(" of them") {
        let num_part = text.trim_end_matches(" of them");
        if let Ok(n) = num_part.parse::<usize>() {
            Condition::Count(n)
        } else {
            Condition::Custom(text)
        }
    } else {
        Condition::Custom(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yara_rule_builder() {
        let rule = YaraRuleBuilder::new("test_rule")
            .meta_string("author", "test")
            .hex_string("pattern1", "48 8B ?? ?? 89")
            .condition_any()
            .tag("malware")
            .build();

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.strings.len(), 1);
        assert_eq!(rule.tags.len(), 1);
    }

    #[test]
    fn test_yara_hex_parse() {
        let pattern = parse_yara_hex("48 8B ?? 89").unwrap();
        assert_eq!(pattern.len(), 4);
    }

    #[test]
    fn test_yara_to_string() {
        let rule = YaraRuleBuilder::new("example")
            .text_string("s1", "test")
            .hex_string("h1", "90 90 90")
            .condition_all()
            .build();

        let yara = rule.to_yara();
        assert!(yara.contains("rule example"));
        assert!(yara.contains("$s1"));
        assert!(yara.contains("$h1"));
    }
}
