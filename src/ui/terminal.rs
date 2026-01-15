// Tue Jan 13 2026 - Alex

use colored::*;
use std::io::{self, Write, BufRead};

pub struct TerminalUI {
    use_color: bool,
    use_unicode: bool,
    width: usize,
    quiet: bool,
}

impl TerminalUI {
    pub fn new() -> Self {
        let width = terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80);

        Self {
            use_color: true,
            use_unicode: true,
            width,
            quiet: false,
        }
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        if !use_color {
            colored::control::set_override(false);
        }
        self
    }

    pub fn with_unicode(mut self, use_unicode: bool) -> Self {
        self.use_unicode = use_unicode;
        self
    }

    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn print_info(&self, message: &str) {
        if self.quiet {
            return;
        }
        if self.use_color {
            println!("{} {}", "[INFO]".cyan().bold(), message);
        } else {
            println!("[INFO] {}", message);
        }
    }

    pub fn print_success(&self, message: &str) {
        if self.quiet {
            return;
        }
        let prefix = if self.use_unicode { "✓" } else { "[OK]" };
        if self.use_color {
            println!("{} {}", prefix.green().bold(), message.green());
        } else {
            println!("{} {}", prefix, message);
        }
    }

    pub fn print_warning(&self, message: &str) {
        if self.quiet {
            return;
        }
        let prefix = if self.use_unicode { "⚠" } else { "[WARN]" };
        if self.use_color {
            println!("{} {}", prefix.yellow().bold(), message.yellow());
        } else {
            println!("{} {}", prefix, message);
        }
    }

    pub fn print_error(&self, message: &str) {
        let prefix = if self.use_unicode { "✗" } else { "[ERROR]" };
        if self.use_color {
            eprintln!("{} {}", prefix.red().bold(), message.red());
        } else {
            eprintln!("{} {}", prefix, message);
        }
    }

    pub fn print_debug(&self, message: &str) {
        if self.quiet {
            return;
        }
        if self.use_color {
            println!("{} {}", "[DEBUG]".dimmed(), message.dimmed());
        } else {
            println!("[DEBUG] {}", message);
        }
    }

    pub fn print_section(&self, title: &str) {
        if self.quiet {
            return;
        }
        let line = if self.use_unicode { "─" } else { "-" };
        let divider = line.repeat(self.width.min(80));

        println!();
        if self.use_color {
            println!("{}", divider.cyan());
            println!("{}", title.cyan().bold());
            println!("{}", divider.cyan());
        } else {
            println!("{}", divider);
            println!("{}", title);
            println!("{}", divider);
        }
    }

    pub fn print_subsection(&self, title: &str) {
        if self.quiet {
            return;
        }
        if self.use_color {
            println!("\n{}", title.yellow().bold());
        } else {
            println!("\n{}", title);
        }
    }

    pub fn print_key_value(&self, key: &str, value: &str) {
        if self.quiet {
            return;
        }
        if self.use_color {
            println!("  {}: {}", key.cyan(), value);
        } else {
            println!("  {}: {}", key, value);
        }
    }

    pub fn print_list_item(&self, item: &str) {
        if self.quiet {
            return;
        }
        let bullet = if self.use_unicode { "•" } else { "-" };
        if self.use_color {
            println!("  {} {}", bullet.cyan(), item);
        } else {
            println!("  {} {}", bullet, item);
        }
    }

    pub fn print_numbered_list(&self, items: &[&str]) {
        if self.quiet {
            return;
        }
        for (i, item) in items.iter().enumerate() {
            if self.use_color {
                println!("  {}. {}", (i + 1).to_string().cyan(), item);
            } else {
                println!("  {}. {}", i + 1, item);
            }
        }
    }

    pub fn print_progress_text(&self, current: usize, total: usize, message: &str) {
        if self.quiet {
            return;
        }
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0) as usize
        } else {
            0
        };

        print!("\r");
        if self.use_color {
            print!("{} [{}/{}] {}%",
                message.cyan(),
                current.to_string().green(),
                total.to_string().cyan(),
                percent.to_string().yellow()
            );
        } else {
            print!("{} [{}/{}] {}%", message, current, total, percent);
        }
        io::stdout().flush().unwrap();
    }

    pub fn print_address(&self, label: &str, address: u64) {
        if self.quiet {
            return;
        }
        if self.use_color {
            println!("  {}: {}", label.cyan(), format!("0x{:016x}", address).red());
        } else {
            println!("  {}: 0x{:016x}", label, address);
        }
    }

    pub fn print_hex_dump(&self, data: &[u8], base_address: u64, bytes_per_line: usize) {
        if self.quiet || data.is_empty() {
            return;
        }

        for (i, chunk) in data.chunks(bytes_per_line).enumerate() {
            let addr = base_address + (i * bytes_per_line) as u64;

            let hex_part: String = chunk.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let ascii_part: String = chunk.iter()
                .map(|&b| if b >= 0x20 && b < 0x7f { b as char } else { '.' })
                .collect();

            let padding = " ".repeat((bytes_per_line - chunk.len()) * 3);

            if self.use_color {
                println!("{} {} {} |{}|",
                    format!("{:016x}", addr).cyan(),
                    hex_part.yellow(),
                    padding,
                    ascii_part.dimmed()
                );
            } else {
                println!("{:016x} {} {} |{}|", addr, hex_part, padding, ascii_part);
            }
        }
    }

    pub fn confirm(&self, message: &str) -> bool {
        if self.use_color {
            print!("{} {} ", message.yellow(), "[y/N]".dimmed());
        } else {
            print!("{} [y/N] ", message);
        }
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();

        matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
    }

    pub fn prompt(&self, message: &str) -> String {
        if self.use_color {
            print!("{} ", message.cyan());
        } else {
            print!("{} ", message);
        }
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();

        line.trim().to_string()
    }

    pub fn clear_line(&self) {
        print!("\r{}\r", " ".repeat(self.width));
        io::stdout().flush().unwrap();
    }

    pub fn newline(&self) {
        println!();
    }

    pub fn horizontal_rule(&self) {
        if self.quiet {
            return;
        }
        let char = if self.use_unicode { "─" } else { "-" };
        let line = char.repeat(self.width.min(80));
        if self.use_color {
            println!("{}", line.dimmed());
        } else {
            println!("{}", line);
        }
    }

    pub fn print_box(&self, title: &str, content: &[&str]) {
        if self.quiet {
            return;
        }

        let (tl, tr, bl, br, h, v) = if self.use_unicode {
            ("┌", "┐", "└", "┘", "─", "│")
        } else {
            ("+", "+", "+", "+", "-", "|")
        };

        let max_len = content.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0)
            .max(title.len());

        let width = max_len + 4;
        let h_line = h.repeat(width - 2);

        if self.use_color {
            println!("{}{}{}", tl.cyan(), h_line.cyan(), tr.cyan());
            println!("{} {:<width$} {}", v.cyan(), title.cyan().bold(), v.cyan(), width = max_len);
            println!("{}{}{}", v.cyan(), h_line.cyan(), v.cyan());
            for line in content {
                println!("{} {:<width$} {}", v.cyan(), line, v.cyan(), width = max_len);
            }
            println!("{}{}{}", bl.cyan(), h_line.cyan(), br.cyan());
        } else {
            println!("{}{}{}", tl, h_line, tr);
            println!("{} {:<width$} {}", v, title, v, width = max_len);
            println!("{}{}{}", v, h_line, v);
            for line in content {
                println!("{} {:<width$} {}", v, line, v, width = max_len);
            }
            println!("{}{}{}", bl, h_line, br);
        }
    }

    pub fn print_stats_box(&self, title: &str, stats: &[(&str, String)]) {
        if self.quiet {
            return;
        }

        self.print_section(title);
        for (key, value) in stats {
            self.print_key_value(key, value);
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

impl Default for TerminalUI {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_terminal_ui() -> TerminalUI {
    TerminalUI::new()
}

pub fn print_banner() {
    let banner = r#"
  ____       _     _             ___   __  __          _   
 |  _ \ ___ | |__ | | _____  __ / _ \ / _|/ _|___  ___| |_ 
 | |_) / _ \| '_ \| |/ _ \ \/ /| | | | |_| |_/ __|/ _ \ __|
 |  _ < (_) | |_) | | (_) >  < | |_| |  _|  _\__ \  __/ |_ 
 |_| \_\___/|_.__/|_|\___/_/\_\ \___/|_| |_| |___/\___|\__|
                                                           
    "#;

    println!("{}", banner.cyan().bold());
    println!("{}", "ARM64 Offset Generator v1.0.0".green());
    println!();
}
