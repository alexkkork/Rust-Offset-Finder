// Tue Jan 13 2026 - Alex

use std::borrow::Cow;

pub struct StringUtils;

impl StringUtils {
    pub fn truncate(s: &str, max_len: usize) -> Cow<'_, str> {
        if s.len() <= max_len {
            Cow::Borrowed(s)
        } else if max_len >= 3 {
            Cow::Owned(format!("{}...", &s[..max_len - 3]))
        } else {
            Cow::Owned(s[..max_len].to_string())
        }
    }

    pub fn pad_left(s: &str, width: usize, pad_char: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            let padding = pad_char.to_string().repeat(width - s.len());
            format!("{}{}", padding, s)
        }
    }

    pub fn pad_right(s: &str, width: usize, pad_char: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            let padding = pad_char.to_string().repeat(width - s.len());
            format!("{}{}", s, padding)
        }
    }

    pub fn center(s: &str, width: usize, pad_char: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            let total_padding = width - s.len();
            let left_padding = total_padding / 2;
            let right_padding = total_padding - left_padding;
            let left = pad_char.to_string().repeat(left_padding);
            let right = pad_char.to_string().repeat(right_padding);
            format!("{}{}{}", left, s, right)
        }
    }

    pub fn snake_case(s: &str) -> String {
        let mut result = String::new();
        let mut prev_was_upper = false;

        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && !prev_was_upper {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                prev_was_upper = true;
            } else if c == ' ' || c == '-' {
                result.push('_');
                prev_was_upper = false;
            } else {
                result.push(c);
                prev_was_upper = false;
            }
        }

        result
    }

    pub fn camel_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for c in s.chars() {
            if c == '_' || c == ' ' || c == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn pascal_case(s: &str) -> String {
        let camel = Self::camel_case(s);
        let mut chars = camel.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    }

    pub fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let first = chars.next().unwrap();

        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        chars.all(|c| c.is_alphanumeric() || c == '_')
    }

    pub fn sanitize_identifier(s: &str) -> String {
        let mut result = String::new();

        for c in s.chars() {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
            } else {
                result.push('_');
            }
        }

        if result.is_empty() || result.chars().next().unwrap().is_numeric() {
            result = format!("_{}", result);
        }

        result
    }

    pub fn escape_string(s: &str) -> String {
        let mut result = String::new();

        for c in s.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\0' => result.push_str("\\0"),
                c if c.is_control() => result.push_str(&format!("\\x{:02x}", c as u32)),
                c => result.push(c),
            }
        }

        result
    }

    pub fn unescape_string(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('0') => result.push('\0'),
                    Some('x') => {
                        let hex: String = chars.by_ref().take(2).collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                        }
                    }
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn levenshtein_distance(a: &str, b: &str) -> usize {
        if a.is_empty() {
            return b.len();
        }
        if b.is_empty() {
            return a.len();
        }

        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        let mut prev_row: Vec<usize> = (0..=b_chars.len()).collect();
        let mut curr_row: Vec<usize> = vec![0; b_chars.len() + 1];

        for (i, a_char) in a_chars.iter().enumerate() {
            curr_row[0] = i + 1;

            for (j, b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };

                curr_row[j + 1] = (prev_row[j + 1] + 1)
                    .min(curr_row[j] + 1)
                    .min(prev_row[j] + cost);
            }

            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        prev_row[b_chars.len()]
    }

    pub fn similarity(a: &str, b: &str) -> f64 {
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return 1.0;
        }

        let distance = Self::levenshtein_distance(a, b);
        1.0 - (distance as f64 / max_len as f64)
    }

    pub fn starts_with_any(s: &str, prefixes: &[&str]) -> bool {
        prefixes.iter().any(|p| s.starts_with(p))
    }

    pub fn ends_with_any(s: &str, suffixes: &[&str]) -> bool {
        suffixes.iter().any(|s2| s.ends_with(s2))
    }

    pub fn contains_any(s: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|p| s.contains(p))
    }

    pub fn split_at_first(s: &str, pattern: &str) -> (String, String) {
        match s.find(pattern) {
            Some(pos) => (s[..pos].to_string(), s[pos + pattern.len()..].to_string()),
            None => (s.to_string(), String::new()),
        }
    }

    pub fn split_at_last(s: &str, pattern: &str) -> (String, String) {
        match s.rfind(pattern) {
            Some(pos) => (s[..pos].to_string(), s[pos + pattern.len()..].to_string()),
            None => (s.to_string(), String::new()),
        }
    }

    pub fn is_printable_ascii(s: &str) -> bool {
        s.bytes().all(|b| b >= 0x20 && b < 0x7F)
    }

    pub fn count_occurrences(s: &str, pattern: &str) -> usize {
        if pattern.is_empty() {
            return 0;
        }
        s.matches(pattern).count()
    }
}

pub fn truncate(s: &str, max_len: usize) -> String {
    StringUtils::truncate(s, max_len).into_owned()
}

pub fn snake_case(s: &str) -> String {
    StringUtils::snake_case(s)
}

pub fn camel_case(s: &str) -> String {
    StringUtils::camel_case(s)
}

pub fn sanitize_identifier(s: &str) -> String {
    StringUtils::sanitize_identifier(s)
}

pub fn escape_string(s: &str) -> String {
    StringUtils::escape_string(s)
}

pub fn similarity(a: &str, b: &str) -> f64 {
    StringUtils::similarity(a, b)
}
