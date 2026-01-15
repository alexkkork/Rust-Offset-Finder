// Wed Jan 15 2026 - Alex

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct ProgressSpinner {
    spinner: ProgressBar,
    message: String,
}

impl ProgressSpinner {
    pub fn new(message: &str) -> Self {
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap();

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(style);
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(80));

        Self {
            spinner,
            message: message.to_string(),
        }
    }

    pub fn with_style(message: &str, chars: &str) -> Self {
        let style = ProgressStyle::default_spinner()
            .template(&format!("{{spinner:.cyan}} {{msg}}"))
            .unwrap()
            .tick_strings(&chars.chars().map(|c| c.to_string()).collect::<Vec<_>>().iter().map(|s| s.as_str()).collect::<Vec<_>>());

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(style);
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(80));

        Self {
            spinner,
            message: message.to_string(),
        }
    }

    pub fn dots(message: &str) -> Self {
        Self::with_style(message, "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
    }

    pub fn line(message: &str) -> Self {
        Self::with_style(message, "-\\|/")
    }

    pub fn arrow(message: &str) -> Self {
        Self::with_style(message, "←↖↑↗→↘↓↙")
    }

    pub fn bounce(message: &str) -> Self {
        Self::with_style(message, "⠁⠂⠄⠂")
    }

    pub fn set_message(&self, message: &str) {
        self.spinner.set_message(message.to_string());
    }

    pub fn finish(&self) {
        self.spinner.finish();
    }

    pub fn finish_with_message(&self, message: &str) {
        self.spinner.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.spinner.finish_and_clear();
    }

    pub fn success(&self, message: &str) {
        self.spinner.finish_with_message(format!("✓ {}", message));
    }

    pub fn failure(&self, message: &str) {
        self.spinner.finish_with_message(format!("✗ {}", message));
    }

    pub fn warning(&self, message: &str) {
        self.spinner.finish_with_message(format!("⚠ {}", message));
    }

    pub fn tick(&self) {
        self.spinner.tick();
    }

    pub fn elapsed(&self) -> Duration {
        self.spinner.elapsed()
    }

    pub fn inner(&self) -> &ProgressBar {
        &self.spinner
    }
}

impl Drop for ProgressSpinner {
    fn drop(&mut self) {
        if !self.spinner.is_finished() {
            self.spinner.finish_and_clear();
        }
    }
}
