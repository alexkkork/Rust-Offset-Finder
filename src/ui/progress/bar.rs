// Wed Jan 15 2026 - Alex

use indicatif::{ProgressBar as IndicatifBar, ProgressStyle};
use std::time::Duration;

pub struct ProgressBar {
    bar: IndicatifBar,
    total: u64,
    message: String,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ");

        let bar = IndicatifBar::new(total);
        bar.set_style(style);

        Self {
            bar,
            total,
            message: String::new(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.bar.set_message(message.to_string());
        self
    }

    pub fn with_style(self, template: &str) -> Self {
        let style = ProgressStyle::default_bar()
            .template(template)
            .unwrap()
            .progress_chars("█▓▒░ ");
        self.bar.set_style(style);
        self
    }

    pub fn increment(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    pub fn position(&self) -> u64 {
        self.bar.position()
    }

    pub fn length(&self) -> Option<u64> {
        self.bar.length()
    }

    pub fn finish(&self) {
        self.bar.finish();
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    pub fn abandon(&self) {
        self.bar.abandon();
    }

    pub fn abandon_with_message(&self, message: &str) {
        self.bar.abandon_with_message(message.to_string());
    }

    pub fn reset(&self) {
        self.bar.reset();
    }

    pub fn is_finished(&self) -> bool {
        self.bar.is_finished()
    }

    pub fn enable_steady_tick(&self, interval: Duration) {
        self.bar.enable_steady_tick(interval);
    }

    pub fn disable_steady_tick(&self) {
        self.bar.disable_steady_tick();
    }

    pub fn elapsed(&self) -> Duration {
        self.bar.elapsed()
    }

    pub fn eta(&self) -> Duration {
        self.bar.eta()
    }

    pub fn per_sec(&self) -> f64 {
        self.bar.per_sec()
    }

    pub fn inner(&self) -> &IndicatifBar {
        &self.bar
    }
}

impl From<IndicatifBar> for ProgressBar {
    fn from(bar: IndicatifBar) -> Self {
        let total = bar.length().unwrap_or(0);
        Self {
            bar,
            total,
            message: String::new(),
        }
    }
}
