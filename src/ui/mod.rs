// Wed Jan 15 2026 - Alex

pub mod cli;
pub mod progress;
pub mod terminal;
pub mod errors;
pub mod display;
pub mod theme;
pub mod spinner;
pub mod table;
pub mod banner;

pub use cli::{Args, Command, CommandHandler};
pub use progress::ProgressManager;
pub use terminal::Terminal;
pub use errors::{ErrorDisplay, ErrorHandler};
pub use display::DisplayRenderer;
pub use theme::Theme;
pub use spinner::Spinner;
pub use table::TableBuilder;
pub use banner::Banner;

use std::io::{self, Write};

pub struct UIManager {
    progress: ProgressManager,
    terminal: Terminal,
    theme: Theme,
    verbose: bool,
    quiet: bool,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            progress: ProgressManager::new(),
            terminal: Terminal::new(),
            theme: Theme::default(),
            verbose: false,
            quiet: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn print_banner(&self) {
        if self.quiet {
            return;
        }
        Banner::print_default();
    }

    pub fn info(&self, message: &str) {
        if self.quiet {
            return;
        }
        self.terminal.print_info(message);
    }

    pub fn success(&self, message: &str) {
        if self.quiet {
            return;
        }
        self.terminal.print_success(message);
    }

    pub fn warning(&self, message: &str) {
        if self.quiet {
            return;
        }
        self.terminal.print_warning(message);
    }

    pub fn error(&self, message: &str) {
        self.terminal.print_error(message);
    }

    pub fn debug(&self, message: &str) {
        if self.verbose {
            self.terminal.print_debug(message);
        }
    }

    pub fn create_progress(&self, total: u64, message: &str) -> ProgressHandle {
        self.progress.create(total, message)
    }

    pub fn create_spinner(&self, message: &str) -> Spinner {
        Spinner::new(message)
    }

    pub fn print_table<T: std::fmt::Display>(&self, headers: &[&str], rows: &[Vec<T>]) {
        if self.quiet {
            return;
        }
        let builder = TableBuilder::new()
            .with_headers(headers)
            .with_rows(rows);
        println!("{}", builder.build());
    }

    pub fn print_summary(&self, title: &str, items: &[(&str, String)]) {
        if self.quiet {
            return;
        }
        self.terminal.print_section(title);
        for (key, value) in items {
            println!("  {}: {}", self.theme.highlight(key), value);
        }
    }

    pub fn confirm(&self, message: &str) -> bool {
        if self.quiet {
            return true;
        }
        self.terminal.confirm(message)
    }

    pub fn flush(&self) {
        let _ = io::stdout().flush();
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProgressHandle {
    id: usize,
    current: u64,
    total: u64,
}

impl ProgressHandle {
    pub fn new(id: usize, total: u64) -> Self {
        Self {
            id,
            current: 0,
            total,
        }
    }

    pub fn inc(&mut self, delta: u64) {
        self.current = (self.current + delta).min(self.total);
    }

    pub fn set(&mut self, value: u64) {
        self.current = value.min(self.total);
    }

    pub fn set_message(&mut self, _message: &str) {
    }

    pub fn finish(&mut self) {
        self.current = self.total;
    }

    pub fn finish_with_message(&mut self, _message: &str) {
        self.current = self.total;
    }

    pub fn progress(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f64 / self.total as f64
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current >= self.total
    }
}

pub fn create_ui() -> UIManager {
    UIManager::new()
}

pub fn print_info(message: &str) {
    use colored::Colorize;
    println!("{} {}", "[INFO]".cyan(), message);
}

pub fn print_success(message: &str) {
    use colored::Colorize;
    println!("{} {}", "[OK]".green(), message);
}

pub fn print_warning(message: &str) {
    use colored::Colorize;
    println!("{} {}", "[WARN]".yellow(), message);
}

pub fn print_error(message: &str) {
    use colored::Colorize;
    eprintln!("{} {}", "[ERROR]".red(), message);
}
