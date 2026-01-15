// Wed Jan 15 2026 - Alex

pub mod bar;
pub mod multi;
pub mod spinner;
pub mod tracker;

pub use bar::ProgressBar;
pub use multi::MultiProgress;
pub use spinner::ProgressSpinner;
pub use tracker::ProgressTracker;

use indicatif::{ProgressStyle, ProgressDrawTarget};
use std::time::Duration;

pub struct ProgressManager {
    multi: indicatif::MultiProgress,
    default_style: ProgressStyle,
}

impl ProgressManager {
    pub fn new() -> Self {
        let default_style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ");

        Self {
            multi: indicatif::MultiProgress::new(),
            default_style,
        }
    }

    pub fn create_main_progress(&self, total: u64, message: &str) -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(self.default_style.clone());
        pb.set_message(message.to_string());
        self.multi.add(pb)
    }

    pub fn create_sub_progress(&self, total: u64, message: &str) -> indicatif::ProgressBar {
        let style = ProgressStyle::default_bar()
            .template("  {spinner:.blue} [{bar:30.white/gray}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> ");

        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(style);
        pb.set_message(message.to_string());
        self.multi.add(pb)
    }

    pub fn create_spinner(&self, message: &str) -> indicatif::ProgressBar {
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap();

        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(style);
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        self.multi.add(pb)
    }

    pub fn create_bytes_progress(&self, total: u64, message: &str) -> indicatif::ProgressBar {
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ");

        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(style);
        pb.set_message(message.to_string());
        self.multi.add(pb)
    }

    pub fn hidden() -> Self {
        let mut manager = Self::new();
        manager.multi.set_draw_target(ProgressDrawTarget::hidden());
        manager
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}
