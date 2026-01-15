// Tue Jan 13 2026 - Alex

use colored::*;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub primary: ThemeColor,
    pub secondary: ThemeColor,
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub error: ThemeColor,
    pub info: ThemeColor,
    pub muted: ThemeColor,
    pub highlight: ThemeColor,
    pub address: ThemeColor,
    pub use_unicode: bool,
    pub icons: ThemeIcons,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct ThemeIcons {
    pub success: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub bullet: String,
    pub arrow: String,
    pub progress_filled: String,
    pub progress_empty: String,
    pub spinner: Vec<String>,
}

impl Theme {
    pub fn default() -> Self {
        Self::cyberpunk()
    }

    pub fn cyberpunk() -> Self {
        Self {
            name: "Cyberpunk".to_string(),
            primary: ThemeColor::new(0, 212, 255),
            secondary: ThemeColor::new(255, 107, 107),
            success: ThemeColor::new(78, 205, 196),
            warning: ThemeColor::new(255, 230, 109),
            error: ThemeColor::new(255, 82, 82),
            info: ThemeColor::new(100, 181, 246),
            muted: ThemeColor::new(128, 128, 128),
            highlight: ThemeColor::new(255, 215, 0),
            address: ThemeColor::new(255, 107, 107),
            use_unicode: true,
            icons: ThemeIcons::unicode(),
        }
    }

    pub fn minimal() -> Self {
        Self {
            name: "Minimal".to_string(),
            primary: ThemeColor::new(255, 255, 255),
            secondary: ThemeColor::new(200, 200, 200),
            success: ThemeColor::new(0, 255, 0),
            warning: ThemeColor::new(255, 255, 0),
            error: ThemeColor::new(255, 0, 0),
            info: ThemeColor::new(0, 191, 255),
            muted: ThemeColor::new(128, 128, 128),
            highlight: ThemeColor::new(255, 255, 255),
            address: ThemeColor::new(255, 165, 0),
            use_unicode: false,
            icons: ThemeIcons::ascii(),
        }
    }

    pub fn matrix() -> Self {
        Self {
            name: "Matrix".to_string(),
            primary: ThemeColor::new(0, 255, 0),
            secondary: ThemeColor::new(0, 200, 0),
            success: ThemeColor::new(0, 255, 0),
            warning: ThemeColor::new(200, 255, 0),
            error: ThemeColor::new(255, 50, 50),
            info: ThemeColor::new(0, 200, 100),
            muted: ThemeColor::new(0, 100, 0),
            highlight: ThemeColor::new(150, 255, 150),
            address: ThemeColor::new(0, 255, 100),
            use_unicode: true,
            icons: ThemeIcons::unicode(),
        }
    }

    pub fn ocean() -> Self {
        Self {
            name: "Ocean".to_string(),
            primary: ThemeColor::new(65, 105, 225),
            secondary: ThemeColor::new(100, 149, 237),
            success: ThemeColor::new(32, 178, 170),
            warning: ThemeColor::new(255, 193, 7),
            error: ThemeColor::new(220, 53, 69),
            info: ThemeColor::new(23, 162, 184),
            muted: ThemeColor::new(108, 117, 125),
            highlight: ThemeColor::new(255, 255, 255),
            address: ThemeColor::new(255, 127, 80),
            use_unicode: true,
            icons: ThemeIcons::unicode(),
        }
    }

    pub fn apply_primary(&self, text: &str) -> ColoredString {
        text.truecolor(self.primary.r, self.primary.g, self.primary.b)
    }

    pub fn apply_secondary(&self, text: &str) -> ColoredString {
        text.truecolor(self.secondary.r, self.secondary.g, self.secondary.b)
    }

    pub fn apply_success(&self, text: &str) -> ColoredString {
        text.truecolor(self.success.r, self.success.g, self.success.b)
    }

    pub fn apply_warning(&self, text: &str) -> ColoredString {
        text.truecolor(self.warning.r, self.warning.g, self.warning.b)
    }

    pub fn apply_error(&self, text: &str) -> ColoredString {
        text.truecolor(self.error.r, self.error.g, self.error.b)
    }

    pub fn apply_info(&self, text: &str) -> ColoredString {
        text.truecolor(self.info.r, self.info.g, self.info.b)
    }

    pub fn apply_muted(&self, text: &str) -> ColoredString {
        text.truecolor(self.muted.r, self.muted.g, self.muted.b)
    }

    pub fn apply_address(&self, address: u64) -> ColoredString {
        format!("0x{:016x}", address)
            .truecolor(self.address.r, self.address.g, self.address.b)
    }

    pub fn highlight(&self, text: &str) -> ColoredString {
        text.truecolor(self.highlight.r, self.highlight.g, self.highlight.b).bold()
    }

    pub fn success_icon(&self) -> &str {
        &self.icons.success
    }

    pub fn error_icon(&self) -> &str {
        &self.icons.error
    }

    pub fn warning_icon(&self) -> &str {
        &self.icons.warning
    }

    pub fn info_icon(&self) -> &str {
        &self.icons.info
    }

    pub fn bullet(&self) -> &str {
        &self.icons.bullet
    }

    pub fn arrow(&self) -> &str {
        &self.icons.arrow
    }

    pub fn print_colored(&self, text: &str, color: &ThemeColor) {
        println!("{}", text.truecolor(color.r, color.g, color.b));
    }

    pub fn format_header(&self, text: &str) -> String {
        let line = if self.use_unicode { "═" } else { "=" };
        let divider = line.repeat(text.len() + 4);
        format!("{}\n  {}  \n{}", divider, text, divider)
    }

    pub fn format_section(&self, text: &str) -> String {
        let line = if self.use_unicode { "─" } else { "-" };
        let divider = line.repeat(40);
        format!("{}\n{}\n{}", divider, text, divider)
    }
}

impl ThemeColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

    pub fn cyan() -> Self {
        Self::new(0, 255, 255)
    }

    pub fn yellow() -> Self {
        Self::new(255, 255, 0)
    }

    pub fn magenta() -> Self {
        Self::new(255, 0, 255)
    }
}

impl ThemeIcons {
    pub fn unicode() -> Self {
        Self {
            success: "✓".to_string(),
            error: "✗".to_string(),
            warning: "⚠".to_string(),
            info: "ℹ".to_string(),
            bullet: "•".to_string(),
            arrow: "→".to_string(),
            progress_filled: "█".to_string(),
            progress_empty: "░".to_string(),
            spinner: vec![
                "⠋".to_string(), "⠙".to_string(), "⠹".to_string(),
                "⠸".to_string(), "⠼".to_string(), "⠴".to_string(),
                "⠦".to_string(), "⠧".to_string(), "⠇".to_string(),
                "⠏".to_string(),
            ],
        }
    }

    pub fn ascii() -> Self {
        Self {
            success: "[OK]".to_string(),
            error: "[X]".to_string(),
            warning: "[!]".to_string(),
            info: "[i]".to_string(),
            bullet: "-".to_string(),
            arrow: "->".to_string(),
            progress_filled: "#".to_string(),
            progress_empty: "-".to_string(),
            spinner: vec![
                "|".to_string(), "/".to_string(),
                "-".to_string(), "\\".to_string(),
            ],
        }
    }
}

pub fn get_theme(name: &str) -> Theme {
    match name.to_lowercase().as_str() {
        "cyberpunk" => Theme::cyberpunk(),
        "minimal" => Theme::minimal(),
        "matrix" => Theme::matrix(),
        "ocean" => Theme::ocean(),
        _ => Theme::default(),
    }
}

pub fn list_themes() -> Vec<&'static str> {
    vec!["cyberpunk", "minimal", "matrix", "ocean"]
}
