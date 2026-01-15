// Wed Jan 15 2026 - Alex

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct ProgressTracker {
    stages: Arc<RwLock<HashMap<String, StageProgress>>>,
    start_time: Instant,
    total_items: u64,
    completed_items: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
pub struct StageProgress {
    pub name: String,
    pub total: u64,
    pub completed: u64,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub status: StageStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl ProgressTracker {
    pub fn new(total_items: u64) -> Self {
        Self {
            stages: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            total_items,
            completed_items: Arc::new(RwLock::new(0)),
        }
    }

    pub fn add_stage(&self, name: &str, total: u64) {
        let mut stages = self.stages.write().unwrap();
        stages.insert(name.to_string(), StageProgress {
            name: name.to_string(),
            total,
            completed: 0,
            started_at: Instant::now(),
            finished_at: None,
            status: StageStatus::Pending,
            message: String::new(),
        });
    }

    pub fn start_stage(&self, name: &str) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.status = StageStatus::Running;
            stage.started_at = Instant::now();
        }
    }

    pub fn update_stage(&self, name: &str, completed: u64, message: Option<&str>) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.completed = completed;
            if let Some(msg) = message {
                stage.message = msg.to_string();
            }
        }
    }

    pub fn increment_stage(&self, name: &str, delta: u64) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.completed += delta;
        }
    }

    pub fn complete_stage(&self, name: &str) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.status = StageStatus::Completed;
            stage.finished_at = Some(Instant::now());
            stage.completed = stage.total;
        }
    }

    pub fn fail_stage(&self, name: &str, message: &str) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.status = StageStatus::Failed;
            stage.finished_at = Some(Instant::now());
            stage.message = message.to_string();
        }
    }

    pub fn skip_stage(&self, name: &str, reason: &str) {
        let mut stages = self.stages.write().unwrap();
        if let Some(stage) = stages.get_mut(name) {
            stage.status = StageStatus::Skipped;
            stage.finished_at = Some(Instant::now());
            stage.message = reason.to_string();
        }
    }

    pub fn get_stage(&self, name: &str) -> Option<StageProgress> {
        let stages = self.stages.read().unwrap();
        stages.get(name).cloned()
    }

    pub fn increment_completed(&self, delta: u64) {
        let mut completed = self.completed_items.write().unwrap();
        *completed += delta;
    }

    pub fn completed_items(&self) -> u64 {
        *self.completed_items.read().unwrap()
    }

    pub fn total_items(&self) -> u64 {
        self.total_items
    }

    pub fn progress_percent(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        let completed = *self.completed_items.read().unwrap();
        (completed as f64 / self.total_items as f64) * 100.0
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn eta(&self) -> Option<Duration> {
        let completed = *self.completed_items.read().unwrap();
        if completed == 0 {
            return None;
        }

        let elapsed = self.start_time.elapsed();
        let rate = completed as f64 / elapsed.as_secs_f64();

        if rate <= 0.0 {
            return None;
        }

        let remaining = self.total_items - completed;
        let eta_secs = remaining as f64 / rate;

        Some(Duration::from_secs_f64(eta_secs))
    }

    pub fn summary(&self) -> TrackerSummary {
        let stages = self.stages.read().unwrap();
        let completed = *self.completed_items.read().unwrap();

        let completed_stages = stages.values()
            .filter(|s| s.status == StageStatus::Completed)
            .count();

        let failed_stages = stages.values()
            .filter(|s| s.status == StageStatus::Failed)
            .count();

        TrackerSummary {
            total_items: self.total_items,
            completed_items: completed,
            total_stages: stages.len(),
            completed_stages,
            failed_stages,
            elapsed: self.elapsed(),
            eta: self.eta(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackerSummary {
    pub total_items: u64,
    pub completed_items: u64,
    pub total_stages: usize,
    pub completed_stages: usize,
    pub failed_stages: usize,
    pub elapsed: Duration,
    pub eta: Option<Duration>,
}

impl TrackerSummary {
    pub fn format(&self) -> String {
        let eta_str = self.eta
            .map(|d| format!("{:.1}s", d.as_secs_f64()))
            .unwrap_or_else(|| "calculating...".to_string());

        format!(
            "Progress: {}/{} items ({:.1}%) | Stages: {}/{} | Elapsed: {:.1}s | ETA: {}",
            self.completed_items,
            self.total_items,
            self.completed_items as f64 / self.total_items.max(1) as f64 * 100.0,
            self.completed_stages,
            self.total_stages,
            self.elapsed.as_secs_f64(),
            eta_str
        )
    }
}
