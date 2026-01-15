// Tue Jan 13 2026 - Alex

use crate::engine::task::{Task, TaskPriority};
use crate::engine::result::TaskResult;
use crate::engine::worker::{Worker, PrioritizedTask};
use std::collections::BinaryHeap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

pub struct TaskScheduler {
    task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
    result_sender: Sender<TaskResult>,
    result_receiver: Receiver<TaskResult>,
    workers: Vec<Worker>,
    thread_count: usize,
    running: Arc<RwLock<bool>>,
}


impl TaskScheduler {
    pub fn new(thread_count: usize) -> Self {
        let (result_sender, result_receiver) = channel();

        Self {
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            result_sender,
            result_receiver,
            workers: Vec::with_capacity(thread_count),
            thread_count,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn start(&mut self) {
        *self.running.write() = true;

        for i in 0..self.thread_count {
            let worker = Worker::new(
                i,
                self.task_queue.clone(),
                self.result_sender.clone(),
                self.running.clone(),
            );
            self.workers.push(worker);
        }

        for worker in &mut self.workers {
            worker.start();
        }
    }

    pub fn stop(&mut self) {
        *self.running.write() = false;

        for worker in &mut self.workers {
            worker.stop();
        }

        self.workers.clear();
    }

    pub fn submit(&self, task: Task) {
        let priority = match task.priority() {
            TaskPriority::Critical => 100,
            TaskPriority::High => 75,
            TaskPriority::Normal => 50,
            TaskPriority::Low => 25,
        };

        let prioritized = PrioritizedTask { task, priority };

        let mut queue = self.task_queue.lock().unwrap();
        queue.push(prioritized);
    }

    pub fn submit_batch(&self, tasks: Vec<Task>) {
        let mut queue = self.task_queue.lock().unwrap();

        for task in tasks {
            let priority = match task.priority() {
                TaskPriority::Critical => 100,
                TaskPriority::High => 75,
                TaskPriority::Normal => 50,
                TaskPriority::Low => 25,
            };

            queue.push(PrioritizedTask { task, priority });
        }
    }

    pub fn wait_for_completion(&self) -> Vec<TaskResult> {
        let mut results = Vec::new();

        while let Ok(result) = self.result_receiver.recv() {
            results.push(result);

            let queue = self.task_queue.lock().unwrap();
            if queue.is_empty() {
                break;
            }
        }

        results
    }

    pub fn pending_count(&self) -> usize {
        let queue = self.task_queue.lock().unwrap();
        queue.len()
    }

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    pub fn worker_count(&self) -> usize {
        self.thread_count
    }

    pub fn clear_queue(&self) {
        let mut queue = self.task_queue.lock().unwrap();
        queue.clear();
    }
}

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct SchedulerStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub skipped_tasks: usize,
    pub average_task_time_ms: f64,
}

impl SchedulerStats {
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            skipped_tasks: 0,
            average_task_time_ms: 0.0,
        }
    }

    pub fn record_completion(&mut self, duration_ms: f64) {
        self.completed_tasks += 1;
        self.total_tasks += 1;

        let n = self.completed_tasks as f64;
        self.average_task_time_ms = ((n - 1.0) * self.average_task_time_ms + duration_ms) / n;
    }

    pub fn record_failure(&mut self) {
        self.failed_tasks += 1;
        self.total_tasks += 1;
    }

    pub fn record_skip(&mut self) {
        self.skipped_tasks += 1;
        self.total_tasks += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed_tasks as f64 / self.total_tasks as f64
        }
    }
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self::new()
    }
}
