// Wed Jan 15 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
    Ansi256(u8),
    Reset,
}

impl TerminalColor {
    pub fn fg_code(&self) -> String {
        match self {
            TerminalColor::Black => "\x1B[30m".to_string(),
            TerminalColor::Red => "\x1B[31m".to_string(),
            TerminalColor::Green => "\x1B[32m".to_string(),
            TerminalColor::Yellow => "\x1B[33m".to_string(),
            TerminalColor::Blue => "\x1B[34m".to_string(),
            TerminalColor::Magenta => "\x1B[35m".to_string(),
            TerminalColor::Cyan => "\x1B[36m".to_string(),
            TerminalColor::White => "\x1B[37m".to_string(),
            TerminalColor::BrightBlack => "\x1B[90m".to_string(),
            TerminalColor::BrightRed => "\x1B[91m".to_string(),
            TerminalColor::BrightGreen => "\x1B[92m".to_string(),
            TerminalColor::BrightYellow => "\x1B[93m".to_string(),
            TerminalColor::BrightBlue => "\x1B[94m".to_string(),
            TerminalColor::BrightMagenta => "\x1B[95m".to_string(),
            TerminalColor::BrightCyan => "\x1B[96m".to_string(),
            TerminalColor::BrightWhite => "\x1B[97m".to_string(),
            TerminalColor::Rgb(r, g, b) => format!("\x1B[38;2;{};{};{}m", r, g, b),
            TerminalColor::Ansi256(n) => format!("\x1B[38;5;{}m", n),
            TerminalColor::Reset => "\x1B[0m".to_string(),
        }
    }

    pub fn bg_code(&self) -> String {
        match self {
            TerminalColor::Black => "\x1B[40m".to_string(),
            TerminalColor::Red => "\x1B[41m".to_string(),
            TerminalColor::Green => "\x1B[42m".to_string(),
            TerminalColor::Yellow => "\x1B[43m".to_string(),
            TerminalColor::Blue => "\x1B[44m".to_string(),
            TerminalColor::Magenta => "\x1B[45m".to_string(),
            TerminalColor::Cyan => "\x1B[46m".to_string(),
            TerminalColor::White => "\x1B[47m".to_string(),
            TerminalColor::BrightBlack => "\x1B[100m".to_string(),
            TerminalColor::BrightRed => "\x1B[101m".to_string(),
            TerminalColor::BrightGreen => "\x1B[102m".to_string(),
            TerminalColor::BrightYellow => "\x1B[103m".to_string(),
            TerminalColor::BrightBlue => "\x1B[104m".to_string(),
            TerminalColor::BrightMagenta => "\x1B[105m".to_string(),
            TerminalColor::BrightCyan => "\x1B[106m".to_string(),
            TerminalColor::BrightWhite => "\x1B[107m".to_string(),
            TerminalColor::Rgb(r, g, b) => format!("\x1B[48;2;{};{};{}m", r, g, b),
            TerminalColor::Ansi256(n) => format!("\x1B[48;5;{}m", n),
            TerminalColor::Reset => "\x1B[0m".to_string(),
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(TerminalColor::Rgb(r, g, b))
    }

    pub fn to_hex(&self) -> Option<String> {
        match self {
            TerminalColor::Rgb(r, g, b) => Some(format!("#{:02X}{:02X}{:02X}", r, g, b)),
            _ => None,
        }
    }
}

pub struct ColoredString {
    text: String,
    fg: Option<TerminalColor>,
    bg: Option<TerminalColor>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl ColoredString {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn fg(mut self, color: TerminalColor) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: TerminalColor) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

impl fmt::Display for ColoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut codes = Vec::new();

        if self.bold {
            codes.push("\x1B[1m".to_string());
        }
        if self.italic {
            codes.push("\x1B[3m".to_string());
        }
        if self.underline {
            codes.push("\x1B[4m".to_string());
        }
        if let Some(ref fg) = self.fg {
            codes.push(fg.fg_code());
        }
        if let Some(ref bg) = self.bg {
            codes.push(bg.bg_code());
        }

        write!(f, "{}{}\x1B[0m", codes.join(""), self.text)
    }
}

pub fn colorize(text: &str) -> ColoredString {
    ColoredString::new(text)
}
