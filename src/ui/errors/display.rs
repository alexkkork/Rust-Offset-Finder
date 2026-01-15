// Wed Jan 15 2026 - Alex

use colored::Colorize;
use std::error::Error;

pub struct ErrorDisplay {
    show_backtrace: bool,
    show_cause_chain: bool,
    color_enabled: bool,
}

impl ErrorDisplay {
    pub fn new() -> Self {
        Self {
            show_backtrace: false,
            show_cause_chain: true,
            color_enabled: true,
        }
    }

    pub fn with_backtrace(mut self) -> Self {
        self.show_backtrace = true;
        self
    }

    pub fn without_colors(mut self) -> Self {
        self.color_enabled = false;
        self
    }

    pub fn format<E: Error>(&self, error: &E) -> String {
        let mut output = String::new();

        let header = if self.color_enabled {
            "Error:".red().bold().to_string()
        } else {
            "Error:".to_string()
        };

        output.push_str(&format!("{} {}\n", header, error));

        if self.show_cause_chain {
            let mut source = error.source();
            let mut depth = 1;

            while let Some(cause) = source {
                let prefix = if self.color_enabled {
                    format!("  {} ", "→".yellow())
                } else {
                    format!("  {} ", "->")
                };

                output.push_str(&format!("{}Caused by: {}\n", prefix, cause));
                source = cause.source();
                depth += 1;

                if depth > 10 {
                    output.push_str("  ... (cause chain truncated)\n");
                    break;
                }
            }
        }

        output
    }

    pub fn print<E: Error>(&self, error: &E) {
        eprintln!("{}", self.format(error));
    }

    pub fn format_simple<E: Error>(&self, error: &E) -> String {
        if self.color_enabled {
            format!("{} {}", "Error:".red(), error)
        } else {
            format!("Error: {}", error)
        }
    }

    pub fn format_warning(&self, message: &str) -> String {
        if self.color_enabled {
            format!("{} {}", "Warning:".yellow().bold(), message)
        } else {
            format!("Warning: {}", message)
        }
    }

    pub fn format_hint(&self, message: &str) -> String {
        if self.color_enabled {
            format!("{} {}", "Hint:".cyan(), message)
        } else {
            format!("Hint: {}", message)
        }
    }

    pub fn format_context(&self, context: &str, message: &str) -> String {
        if self.color_enabled {
            format!("{}: {}", context.blue().bold(), message)
        } else {
            format!("{}: {}", context, message)
        }
    }
}

impl Default for ErrorDisplay {
    fn default() -> Self {
        Self::new()
    }
}
