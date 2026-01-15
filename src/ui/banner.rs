// Tue Jan 13 2026 - Alex

use colored::*;

pub struct Banner {
    title: String,
    subtitle: Option<String>,
    version: Option<String>,
    author: Option<String>,
    style: BannerStyle,
    use_color: bool,
    width: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerStyle {
    Simple,
    Box,
    Fancy,
    Minimal,
    Cyberpunk,
}

impl Banner {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            subtitle: None,
            version: None,
            author: None,
            style: BannerStyle::Fancy,
            use_color: true,
            width: 60,
        }
    }

    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }

    pub fn with_style(mut self, style: BannerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn render(&self) -> String {
        match self.style {
            BannerStyle::Simple => self.render_simple(),
            BannerStyle::Box => self.render_box(),
            BannerStyle::Fancy => self.render_fancy(),
            BannerStyle::Minimal => self.render_minimal(),
            BannerStyle::Cyberpunk => self.render_cyberpunk(),
        }
    }

    pub fn print(&self) {
        println!("{}", self.render());
    }

    fn render_simple(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== {} ===", self.title));

        if let Some(subtitle) = &self.subtitle {
            lines.push(subtitle.clone());
        }

        if let Some(version) = &self.version {
            lines.push(format!("Version: {}", version));
        }

        lines.join("\n")
    }

    fn render_box(&self) -> String {
        let mut lines = Vec::new();
        let inner_width = self.width - 4;

        let h_line = "─".repeat(inner_width + 2);
        lines.push(format!("┌{}┐", h_line));

        let title_line = format!("{:^width$}", self.title, width = inner_width);
        if self.use_color {
            lines.push(format!("│ {} │", title_line.cyan().bold()));
        } else {
            lines.push(format!("│ {} │", title_line));
        }

        if let Some(subtitle) = &self.subtitle {
            let sub_line = format!("{:^width$}", subtitle, width = inner_width);
            lines.push(format!("│ {} │", sub_line));
        }

        if self.version.is_some() || self.author.is_some() {
            lines.push(format!("├{}┤", h_line));

            if let Some(version) = &self.version {
                let ver_line = format!("{:^width$}", format!("v{}", version), width = inner_width);
                if self.use_color {
                    lines.push(format!("│ {} │", ver_line.green()));
                } else {
                    lines.push(format!("│ {} │", ver_line));
                }
            }

            if let Some(author) = &self.author {
                let auth_line = format!("{:^width$}", format!("by {}", author), width = inner_width);
                lines.push(format!("│ {} │", auth_line));
            }
        }

        lines.push(format!("└{}┘", h_line));

        lines.join("\n")
    }

    fn render_fancy(&self) -> String {
        let ascii_art = r#"
  ____       _     _             ___  __  __          _
 |  _ \ ___ | |__ | | _____  __ / _ \/ _|/ _|___  ___| |_
 | |_) / _ \| '_ \| |/ _ \ \/ /| | | | |_| |_/ __|/ _ \ __|
 |  _ < (_) | |_) | | (_) >  < | |_| |  _|  _\__ \  __/ |_
 |_| \_\___/|_.__/|_|\___/_/\_\ \___/|_| |_| |___/\___|\__|
        "#;

        let mut lines = Vec::new();

        if self.use_color {
            for line in ascii_art.lines() {
                lines.push(line.cyan().bold().to_string());
            }
        } else {
            lines.push(ascii_art.to_string());
        }

        lines.push(String::new());

        if let Some(subtitle) = &self.subtitle {
            let centered = format!("{:^60}", subtitle);
            if self.use_color {
                lines.push(centered.yellow().to_string());
            } else {
                lines.push(centered);
            }
        }

        let version_str = self.version.as_ref().map(|v| format!("v{}", v)).unwrap_or_default();
        let author_str = self.author.as_ref().map(|a| format!("by {}", a)).unwrap_or_default();

        if !version_str.is_empty() || !author_str.is_empty() {
            let info = format!("{} {}", version_str, author_str).trim().to_string();
            let centered = format!("{:^60}", info);
            if self.use_color {
                lines.push(centered.green().to_string());
            } else {
                lines.push(centered);
            }
        }

        lines.push(String::new());

        lines.join("\n")
    }

    fn render_minimal(&self) -> String {
        let mut lines = Vec::new();

        if self.use_color {
            lines.push(self.title.cyan().bold().to_string());
        } else {
            lines.push(self.title.clone());
        }

        if let Some(subtitle) = &self.subtitle {
            if self.use_color {
                lines.push(subtitle.dimmed().to_string());
            } else {
                lines.push(subtitle.clone());
            }
        }

        lines.join("\n")
    }

    fn render_cyberpunk(&self) -> String {
        let mut lines = Vec::new();

        let cyber_box = format!(
            "╔{}╗",
            "═".repeat(self.width - 2)
        );
        let cyber_bottom = format!(
            "╚{}╝",
            "═".repeat(self.width - 2)
        );

        if self.use_color {
            lines.push(cyber_box.truecolor(0, 212, 255).to_string());
        } else {
            lines.push(cyber_box);
        }

        let title_display = format!("║ {:^width$} ║", self.title, width = self.width - 4);
        if self.use_color {
            let colored_title = format!("║ {} ║",
                format!("{:^width$}", self.title, width = self.width - 4)
                    .truecolor(255, 107, 107)
                    .bold()
            );
            lines.push(colored_title);
        } else {
            lines.push(title_display);
        }

        if let Some(subtitle) = &self.subtitle {
            let sub_display = format!("║ {:^width$} ║", subtitle, width = self.width - 4);
            if self.use_color {
                lines.push(format!("║ {} ║",
                    format!("{:^width$}", subtitle, width = self.width - 4)
                        .truecolor(78, 205, 196)
                ));
            } else {
                lines.push(sub_display);
            }
        }

        let divider = format!("╠{}╣", "═".repeat(self.width - 2));
        if self.use_color {
            lines.push(divider.truecolor(0, 212, 255).to_string());
        } else {
            lines.push(divider);
        }

        if let Some(version) = &self.version {
            let ver_str = format!("VERSION {}", version);
            let ver_display = format!("║ {:^width$} ║", ver_str, width = self.width - 4);
            if self.use_color {
                lines.push(format!("║ {} ║",
                    format!("{:^width$}", ver_str, width = self.width - 4)
                        .truecolor(255, 230, 109)
                ));
            } else {
                lines.push(ver_display);
            }
        }

        if self.use_color {
            lines.push(cyber_bottom.truecolor(0, 212, 255).to_string());
        } else {
            lines.push(cyber_bottom);
        }

        lines.join("\n")
    }

    pub fn print_default() {
        let banner = Banner::new("Roblox Offset Generator")
            .with_subtitle("ARM64 Binary Analysis Tool")
            .with_version("1.0.0")
            .with_author("Alex")
            .with_style(BannerStyle::Fancy);

        banner.print();
    }
}

impl Default for Banner {
    fn default() -> Self {
        Self::new("Roblox Offset Generator")
            .with_subtitle("ARM64 Binary Analysis Tool")
            .with_version("1.0.0")
    }
}

pub fn print_banner() {
    Banner::print_default();
}

pub fn print_simple_banner(title: &str) {
    Banner::new(title)
        .with_style(BannerStyle::Simple)
        .print();
}

pub fn print_cyberpunk_banner() {
    Banner::new("ROBLOX OFFSET GENERATOR")
        .with_subtitle("// NEURAL INTERFACE ACTIVE //")
        .with_version("1.0.0")
        .with_style(BannerStyle::Cyberpunk)
        .print();
}

pub fn create_banner(title: &str) -> Banner {
    Banner::new(title)
}
