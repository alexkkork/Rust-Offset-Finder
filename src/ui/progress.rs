// Tue Jan 13 2026 - Alex

use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct ProgressManager {
    multi: Arc<MultiProgress>,
    bars: Arc<Mutex<HashMap<String, ProgressBar>>>,
    enabled: bool,
    style_template: String,
    spinner_template: String,
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            multi: Arc::new(MultiProgress::new()),
            bars: Arc::new(Mutex::new(HashMap::new())),
            enabled: true,
            style_template: "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}".to_string(),
            spinner_template: "{spinner:.cyan} {msg}".to_string(),
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        if !enabled {
            self.multi.set_draw_target(ProgressDrawTarget::hidden());
        }
        self
    }

    pub fn with_style(mut self, template: &str) -> Self {
        self.style_template = template.to_string();
        self
    }

    pub fn create(&self, total: u64, message: &str) -> crate::ui::ProgressHandle {
        if !self.enabled {
            return crate::ui::ProgressHandle::new(0, total);
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&self.style_template)
                .unwrap()
                .progress_chars("█▓▒░ ")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        let mut bars = self.bars.lock().unwrap();
        let id = bars.len();
        bars.insert(format!("bar_{}", id), pb);

        crate::ui::ProgressHandle::new(id, total)
    }

    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        if !self.enabled {
            let pb = ProgressBar::hidden();
            return pb;
        }

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template(&self.spinner_template)
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        pb
    }

    pub fn create_bytes_progress(&self, total: u64, message: &str) -> ProgressBar {
        if !self.enabled {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
                .unwrap()
                .progress_chars("█▓▒░ ")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        pb
    }

    pub fn create_with_template(&self, total: u64, message: &str, template: &str) -> ProgressBar {
        if !self.enabled {
            return ProgressBar::hidden();
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(template)
                .unwrap()
                .progress_chars("█▓▒░ ")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        pb
    }

    pub fn get_multi(&self) -> Arc<MultiProgress> {
        self.multi.clone()
    }

    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.multi.suspend(f)
    }

    pub fn clear(&self) {
        let mut bars = self.bars.lock().unwrap();
        for (_, pb) in bars.drain() {
            pb.finish_and_clear();
        }
    }

    pub fn println(&self, message: &str) {
        if self.enabled {
            let _ = self.multi.println(message);
        } else {
            println!("{}", message);
        }
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProgressManager {
    fn clone(&self) -> Self {
        Self {
            multi: self.multi.clone(),
            bars: self.bars.clone(),
            enabled: self.enabled,
            style_template: self.style_template.clone(),
            spinner_template: self.spinner_template.clone(),
        }
    }
}

pub struct ScanProgress {
    pub overall: ProgressBar,
    pub pattern_scan: ProgressBar,
    pub symbol_scan: ProgressBar,
    pub xref_analysis: ProgressBar,
    pub heuristics: ProgressBar,
    pub validation: ProgressBar,
}

impl ScanProgress {
    pub fn new(manager: &ProgressManager) -> Self {
        let overall = manager.create_with_template(
            100,
            "Overall Progress",
            "{spinner:.green} [{elapsed_precise}] [{bar:50.green/dark_gray}] {pos}% {msg}"
        );

        let pattern_scan = manager.create_with_template(
            0,
            "Pattern Scanning",
            "{spinner:.cyan} [{bar:30.cyan/dark_gray}] {pos}/{len} {msg}"
        );

        let symbol_scan = manager.create_with_template(
            0,
            "Symbol Matching",
            "{spinner:.yellow} [{bar:30.yellow/dark_gray}] {pos}/{len} {msg}"
        );

        let xref_analysis = manager.create_with_template(
            0,
            "XRef Analysis",
            "{spinner:.magenta} [{bar:30.magenta/dark_gray}] {pos}/{len} {msg}"
        );

        let heuristics = manager.create_with_template(
            0,
            "Heuristics",
            "{spinner:.blue} [{bar:30.blue/dark_gray}] {pos}/{len} {msg}"
        );

        let validation = manager.create_with_template(
            0,
            "Validation",
            "{spinner:.white} [{bar:30.white/dark_gray}] {pos}/{len} {msg}"
        );

        Self {
            overall,
            pattern_scan,
            symbol_scan,
            xref_analysis,
            heuristics,
            validation,
        }
    }

    pub fn set_pattern_total(&self, total: u64) {
        self.pattern_scan.set_length(total);
    }

    pub fn set_symbol_total(&self, total: u64) {
        self.symbol_scan.set_length(total);
    }

    pub fn set_xref_total(&self, total: u64) {
        self.xref_analysis.set_length(total);
    }

    pub fn set_heuristics_total(&self, total: u64) {
        self.heuristics.set_length(total);
    }

    pub fn set_validation_total(&self, total: u64) {
        self.validation.set_length(total);
    }

    pub fn inc_patterns(&self) {
        self.pattern_scan.inc(1);
        self.update_overall();
    }

    pub fn inc_symbols(&self) {
        self.symbol_scan.inc(1);
        self.update_overall();
    }

    pub fn inc_xrefs(&self) {
        self.xref_analysis.inc(1);
        self.update_overall();
    }

    pub fn inc_heuristics(&self) {
        self.heuristics.inc(1);
        self.update_overall();
    }

    pub fn inc_validation(&self) {
        self.validation.inc(1);
        self.update_overall();
    }

    fn update_overall(&self) {
        let total_len = self.pattern_scan.length().unwrap_or(0)
            + self.symbol_scan.length().unwrap_or(0)
            + self.xref_analysis.length().unwrap_or(0)
            + self.heuristics.length().unwrap_or(0)
            + self.validation.length().unwrap_or(0);

        let total_pos = self.pattern_scan.position()
            + self.symbol_scan.position()
            + self.xref_analysis.position()
            + self.heuristics.position()
            + self.validation.position();

        if total_len > 0 {
            let percent = (total_pos as f64 / total_len as f64 * 100.0) as u64;
            self.overall.set_position(percent.min(100));
        }
    }

    pub fn finish_patterns(&self) {
        self.pattern_scan.finish_with_message("Done");
    }

    pub fn finish_symbols(&self) {
        self.symbol_scan.finish_with_message("Done");
    }

    pub fn finish_xrefs(&self) {
        self.xref_analysis.finish_with_message("Done");
    }

    pub fn finish_heuristics(&self) {
        self.heuristics.finish_with_message("Done");
    }

    pub fn finish_validation(&self) {
        self.validation.finish_with_message("Done");
    }

    pub fn finish_all(&self, message: &str) {
        self.pattern_scan.finish_and_clear();
        self.symbol_scan.finish_and_clear();
        self.xref_analysis.finish_and_clear();
        self.heuristics.finish_and_clear();
        self.validation.finish_and_clear();
        self.overall.finish_with_message(message.to_string());
    }
}

pub struct PhaseProgress {
    manager: ProgressManager,
    current_phase: Option<ProgressBar>,
    phases_completed: usize,
    total_phases: usize,
}

impl PhaseProgress {
    pub fn new(manager: ProgressManager, total_phases: usize) -> Self {
        Self {
            manager,
            current_phase: None,
            phases_completed: 0,
            total_phases,
        }
    }

    pub fn start_phase(&mut self, name: &str, total: u64) {
        if let Some(pb) = self.current_phase.take() {
            pb.finish_and_clear();
        }

        let phase_num = self.phases_completed + 1;
        let message = format!("[{}/{}] {}", phase_num, self.total_phases, name);
        self.current_phase = Some(self.manager.create_with_template(
            total,
            &message,
            "{spinner:.cyan} [{bar:40.cyan/dark_gray}] {pos}/{len} {msg}"
        ));
    }

    pub fn inc(&self) {
        if let Some(pb) = &self.current_phase {
            pb.inc(1);
        }
    }

    pub fn set_message(&self, msg: &str) {
        if let Some(pb) = &self.current_phase {
            pb.set_message(msg.to_string());
        }
    }

    pub fn finish_phase(&mut self) {
        if let Some(pb) = self.current_phase.take() {
            pb.finish_with_message("Done");
        }
        self.phases_completed += 1;
    }

    pub fn finish_all(&mut self, message: &str) {
        if let Some(pb) = self.current_phase.take() {
            pb.finish_with_message(message.to_string());
        }
        self.manager.println(&format!("\n✓ All {} phases completed!", self.total_phases));
    }
}

pub fn create_progress_manager() -> ProgressManager {
    ProgressManager::new()
}

pub fn create_simple_progress(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ")
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

pub fn create_simple_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
