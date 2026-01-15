// Tue Jan 13 2026 - Alex

use crate::finders::result::FinderResults;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum TaskResult {
    Success(FinderResults),
    Error(String),
    Skipped(String),
}

impl TaskResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TaskResult::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, TaskResult::Error(_))
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, TaskResult::Skipped(_))
    }

    pub fn unwrap(self) -> FinderResults {
        match self {
            TaskResult::Success(results) => results,
            TaskResult::Error(e) => panic!("Called unwrap on Error: {}", e),
            TaskResult::Skipped(reason) => panic!("Called unwrap on Skipped: {}", reason),
        }
    }

    pub fn unwrap_or_default(self) -> FinderResults {
        match self {
            TaskResult::Success(results) => results,
            _ => FinderResults::new(),
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            TaskResult::Error(e) => Some(e),
            _ => None,
        }
    }

    pub fn skip_reason(&self) -> Option<&str> {
        match self {
            TaskResult::Skipped(reason) => Some(reason),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskResultWithMetadata {
    pub result: TaskResult,
    pub task_id: u64,
    pub task_name: String,
    pub duration: Duration,
    pub retry_count: u32,
}

impl TaskResultWithMetadata {
    pub fn new(result: TaskResult, task_id: u64, task_name: String, duration: Duration) -> Self {
        Self {
            result,
            task_id,
            task_name,
            duration,
            retry_count: 0,
        }
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }

    pub fn is_error(&self) -> bool {
        self.result.is_error()
    }
}

pub struct AggregatedResults {
    pub results: FinderResults,
    pub task_results: Vec<TaskResultWithMetadata>,
    pub total_duration: Duration,
    pub success_count: usize,
    pub error_count: usize,
    pub skipped_count: usize,
}

impl AggregatedResults {
    pub fn new() -> Self {
        Self {
            results: FinderResults::new(),
            task_results: Vec::new(),
            total_duration: Duration::ZERO,
            success_count: 0,
            error_count: 0,
            skipped_count: 0,
        }
    }

    pub fn add(&mut self, task_result: TaskResultWithMetadata) {
        self.total_duration += task_result.duration;

        match &task_result.result {
            TaskResult::Success(findings) => {
                self.success_count += 1;
                self.results.merge(findings.clone());
            }
            TaskResult::Error(_) => {
                self.error_count += 1;
            }
            TaskResult::Skipped(_) => {
                self.skipped_count += 1;
            }
        }

        self.task_results.push(task_result);
    }

    pub fn total_tasks(&self) -> usize {
        self.task_results.len()
    }

    pub fn success_rate(&self) -> f64 {
        if self.task_results.is_empty() {
            0.0
        } else {
            self.success_count as f64 / self.task_results.len() as f64
        }
    }

    pub fn average_duration(&self) -> Duration {
        if self.task_results.is_empty() {
            Duration::ZERO
        } else {
            self.total_duration / self.task_results.len() as u32
        }
    }

    pub fn errors(&self) -> Vec<&TaskResultWithMetadata> {
        self.task_results.iter()
            .filter(|r| r.is_error())
            .collect()
    }

    pub fn slowest_tasks(&self, n: usize) -> Vec<&TaskResultWithMetadata> {
        let mut sorted: Vec<_> = self.task_results.iter().collect();
        sorted.sort_by(|a, b| b.duration.cmp(&a.duration));
        sorted.into_iter().take(n).collect()
    }

    pub fn function_count(&self) -> usize {
        self.results.functions.len()
    }

    pub fn structure_offset_count(&self) -> usize {
        self.results.structure_offsets.values()
            .map(|m| m.len())
            .sum()
    }

    pub fn class_count(&self) -> usize {
        self.results.classes.len()
    }

    pub fn constant_count(&self) -> usize {
        self.results.constants.len()
    }
}

impl Default for AggregatedResults {
    fn default() -> Self {
        Self::new()
    }
}
