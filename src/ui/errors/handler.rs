// Wed Jan 15 2026 - Alex

use super::display::ErrorDisplay;
use super::reporter::ErrorReporter;
use std::error::Error;

pub struct ErrorHandler {
    display: ErrorDisplay,
    reporter: Option<ErrorReporter>,
    exit_on_error: bool,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            display: ErrorDisplay::new(),
            reporter: None,
            exit_on_error: true,
        }
    }

    pub fn with_display(mut self, display: ErrorDisplay) -> Self {
        self.display = display;
        self
    }

    pub fn with_reporter(mut self, reporter: ErrorReporter) -> Self {
        self.reporter = Some(reporter);
        self
    }

    pub fn continue_on_error(mut self) -> Self {
        self.exit_on_error = false;
        self
    }

    pub fn handle<E: Error + 'static>(&self, error: E) {
        self.display.print(&error);

        if let Some(ref reporter) = self.reporter {
            reporter.report(&error);
        }

        if self.exit_on_error {
            std::process::exit(1);
        }
    }

    pub fn handle_result<T, E: Error + 'static>(&self, result: Result<T, E>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.handle(error);
                None
            }
        }
    }

    pub fn try_or_exit<T, E: Error + 'static>(&self, result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.display.print(&error);
                if let Some(ref reporter) = self.reporter {
                    reporter.report(&error);
                }
                std::process::exit(1);
            }
        }
    }

    pub fn warn<E: Error>(&self, error: &E) {
        let warning = self.display.format_warning(&error.to_string());
        eprintln!("{}", warning);
    }

    pub fn hint(&self, message: &str) {
        let hint = self.display.format_hint(message);
        eprintln!("{}", hint);
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        let display = ErrorDisplay::new();

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        eprintln!("{}", display.format_context("Panic", &message));
        eprintln!("{}", display.format_context("Location", &location));

        eprintln!("\n{}", display.format_hint("This is a bug. Please report it."));
    }));
}
