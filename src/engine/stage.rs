// Tue Jan 13 2026 - Alex

use crate::engine::task::Task;
use crate::engine::result::TaskResult;
use std::time::{Duration, Instant};

pub struct Stage {
    name: String,
    tasks: Vec<Task>,
    completed_tasks: Vec<u64>,
    state: StageState,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Stage {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tasks: Vec::new(),
            completed_tasks: Vec::new(),
            state: StageState::Pending,
            start_time: None,
            end_time: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed_tasks.len()
    }

    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            1.0
        } else {
            self.completed_tasks.len() as f64 / self.tasks.len() as f64
        }
    }

    pub fn state(&self) -> StageState {
        self.state
    }

    pub fn is_pending(&self) -> bool {
        self.state == StageState::Pending
    }

    pub fn is_running(&self) -> bool {
        self.state == StageState::Running
    }

    pub fn is_completed(&self) -> bool {
        self.state == StageState::Completed
    }

    pub fn is_failed(&self) -> bool {
        self.state == StageState::Failed
    }

    pub fn start(&mut self) {
        self.state = StageState::Running;
        self.start_time = Some(Instant::now());
    }

    pub fn complete(&mut self) {
        self.state = StageState::Completed;
        self.end_time = Some(Instant::now());
    }

    pub fn fail(&mut self) {
        self.state = StageState::Failed;
        self.end_time = Some(Instant::now());
    }

    pub fn reset(&mut self) {
        self.completed_tasks.clear();
        self.state = StageState::Pending;
        self.start_time = None;
        self.end_time = None;
    }

    pub fn mark_task_completed(&mut self, task_id: u64) {
        if !self.completed_tasks.contains(&task_id) {
            self.completed_tasks.push(task_id);
        }

        if self.completed_tasks.len() == self.tasks.len() {
            self.complete();
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            (Some(start), None) => Some(start.elapsed()),
            _ => None,
        }
    }

    pub fn generate_tasks(&self) -> Vec<Task> {
        self.tasks.clone()
    }

    pub fn pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter()
            .filter(|t| !self.completed_tasks.contains(&t.id()))
            .collect()
    }

    pub fn has_pending_tasks(&self) -> bool {
        self.completed_tasks.len() < self.tasks.len()
    }
}

impl Default for Stage {
    fn default() -> Self {
        Self::new(String::new())
    }
}

pub struct StageResult {
    pub stage_name: String,
    pub task_results: Vec<TaskResult>,
    pub duration: Duration,
    pub success: bool,
}

impl StageResult {
    pub fn new(stage_name: String, task_results: Vec<TaskResult>, duration: Duration) -> Self {
        let success = task_results.iter().all(|r| r.is_success() || r.is_skipped());

        Self {
            stage_name,
            task_results,
            duration,
            success,
        }
    }

    pub fn success_count(&self) -> usize {
        self.task_results.iter().filter(|r| r.is_success()).count()
    }

    pub fn error_count(&self) -> usize {
        self.task_results.iter().filter(|r| r.is_error()).count()
    }

    pub fn skipped_count(&self) -> usize {
        self.task_results.iter().filter(|r| r.is_skipped()).count()
    }

    pub fn total_count(&self) -> usize {
        self.task_results.len()
    }

    pub fn errors(&self) -> Vec<&str> {
        self.task_results.iter()
            .filter_map(|r| r.error_message())
            .collect()
    }
}
