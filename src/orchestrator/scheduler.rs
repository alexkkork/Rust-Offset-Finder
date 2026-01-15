// Tue Jan 13 2026 - Alex

use crate::finders::result::FinderResults;
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

pub struct DiscoveryScheduler {
    tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    workers: Vec<WorkerThread>,
    result_sender: Sender<TaskResult>,
    result_receiver: Receiver<TaskResult>,
    max_workers: usize,
    running: Arc<RwLock<bool>>,
}

impl DiscoveryScheduler {
    pub fn new(max_workers: usize) -> Self {
        let (result_sender, result_receiver) = channel();

        Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            workers: Vec::with_capacity(max_workers),
            result_sender,
            result_receiver,
            max_workers,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn start(&mut self) {
        *self.running.write() = true;

        for id in 0..self.max_workers {
            let worker = WorkerThread::new(
                id,
                self.tasks.clone(),
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

    pub fn schedule(&self, task: DiscoveryTask) {
        let priority = task.priority;
        let scheduled = ScheduledTask {
            task,
            priority,
            scheduled_at: Instant::now(),
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(scheduled);
    }

    pub fn schedule_batch(&self, tasks: Vec<DiscoveryTask>) {
        let mut task_queue = self.tasks.lock().unwrap();

        for task in tasks {
            let priority = task.priority;
            let scheduled = ScheduledTask {
                task,
                priority,
                scheduled_at: Instant::now(),
            };
            task_queue.push(scheduled);
        }
    }

    pub fn collect_results(&self) -> Vec<TaskResult> {
        let mut results = Vec::new();

        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }

        results
    }

    pub fn wait_for_all(&self) -> Vec<TaskResult> {
        let mut results = Vec::new();

        loop {
            let pending = {
                let tasks = self.tasks.lock().unwrap();
                !tasks.is_empty()
            };

            let active_workers = self.workers.iter()
                .filter(|w| w.is_active())
                .count();

            if !pending && active_workers == 0 {
                break;
            }

            while let Ok(result) = self.result_receiver.try_recv() {
                results.push(result);
            }

            thread::sleep(Duration::from_millis(10));
        }

        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }

        results
    }

    pub fn pending_count(&self) -> usize {
        let tasks = self.tasks.lock().unwrap();
        tasks.len()
    }

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn active_worker_count(&self) -> usize {
        self.workers.iter().filter(|w| w.is_active()).count()
    }
}

impl Drop for DiscoveryScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

struct WorkerThread {
    id: usize,
    tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    result_sender: Sender<TaskResult>,
    running: Arc<RwLock<bool>>,
    handle: Option<thread::JoinHandle<()>>,
    active: Arc<RwLock<bool>>,
}

impl WorkerThread {
    fn new(
        id: usize,
        tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
        result_sender: Sender<TaskResult>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            id,
            tasks,
            result_sender,
            running,
            handle: None,
            active: Arc::new(RwLock::new(false)),
        }
    }

    fn start(&mut self) {
        let id = self.id;
        let tasks = self.tasks.clone();
        let result_sender = self.result_sender.clone();
        let running = self.running.clone();
        let active = self.active.clone();

        let handle = thread::spawn(move || {
            WorkerThread::run_loop(id, tasks, result_sender, running, active);
        });

        self.handle = Some(handle);
    }

    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn is_active(&self) -> bool {
        *self.active.read()
    }

    fn run_loop(
        id: usize,
        tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
        result_sender: Sender<TaskResult>,
        running: Arc<RwLock<bool>>,
        active: Arc<RwLock<bool>>,
    ) {
        loop {
            if !*running.read() {
                break;
            }

            let task = {
                let mut task_queue = tasks.lock().unwrap();
                task_queue.pop()
            };

            match task {
                Some(scheduled_task) => {
                    *active.write() = true;
                    let start_time = Instant::now();
                    let task_id = scheduled_task.task.id;
                    let task_name = scheduled_task.task.name.clone();

                    let result = scheduled_task.task.execute();
                    let duration = start_time.elapsed();

                    let task_result = TaskResult {
                        task_id,
                        task_name,
                        results: result,
                        duration,
                        worker_id: id,
                    };

                    let _ = result_sender.send(task_result);
                    *active.write() = false;
                }
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DiscoveryTask {
    pub id: u64,
    pub name: String,
    pub priority: i32,
    pub task_type: DiscoveryTaskType,
    pub dependencies: Vec<u64>,
}

impl DiscoveryTask {
    pub fn new(id: u64, name: &str, task_type: DiscoveryTaskType) -> Self {
        Self {
            id,
            name: name.to_string(),
            priority: 0,
            task_type,
            dependencies: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_dependency(mut self, dep_id: u64) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    pub fn execute(&self) -> Result<FinderResults, String> {
        match &self.task_type {
            DiscoveryTaskType::ScanPattern { pattern, mask } => {
                Ok(FinderResults::new())
            }
            DiscoveryTaskType::AnalyzeFunction { address } => {
                Ok(FinderResults::new())
            }
            DiscoveryTaskType::DiscoverStructure { name } => {
                Ok(FinderResults::new())
            }
            DiscoveryTaskType::ValidateOffset { name, offset } => {
                Ok(FinderResults::new())
            }
            DiscoveryTaskType::Custom { handler } => {
                handler()
            }
        }
    }
}

#[derive(Clone)]
pub enum DiscoveryTaskType {
    ScanPattern { pattern: String, mask: String },
    AnalyzeFunction { address: u64 },
    DiscoverStructure { name: String },
    ValidateOffset { name: String, offset: u64 },
    Custom { handler: fn() -> Result<FinderResults, String> },
}

struct ScheduledTask {
    task: DiscoveryTask,
    priority: i32,
    scheduled_at: Instant,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.scheduled_at.cmp(&self.scheduled_at))
    }
}

#[derive(Debug)]
pub struct TaskResult {
    pub task_id: u64,
    pub task_name: String,
    pub results: Result<FinderResults, String>,
    pub duration: Duration,
    pub worker_id: usize,
}

impl TaskResult {
    pub fn is_success(&self) -> bool {
        self.results.is_ok()
    }

    pub fn is_error(&self) -> bool {
        self.results.is_err()
    }

    pub fn error_message(&self) -> Option<&str> {
        self.results.as_ref().err().map(|s| s.as_str())
    }
}

pub struct SchedulerStatistics {
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub total_duration: Duration,
    pub average_task_duration: Duration,
    pub tasks_per_worker: HashMap<usize, usize>,
}

impl SchedulerStatistics {
    pub fn from_results(results: &[TaskResult]) -> Self {
        let mut tasks_completed = 0;
        let mut tasks_failed = 0;
        let mut total_duration = Duration::ZERO;
        let mut tasks_per_worker: HashMap<usize, usize> = HashMap::new();

        for result in results {
            if result.is_success() {
                tasks_completed += 1;
            } else {
                tasks_failed += 1;
            }

            total_duration += result.duration;
            *tasks_per_worker.entry(result.worker_id).or_insert(0) += 1;
        }

        let average_task_duration = if results.is_empty() {
            Duration::ZERO
        } else {
            total_duration / results.len() as u32
        };

        Self {
            tasks_completed,
            tasks_failed,
            total_duration,
            average_task_duration,
            tasks_per_worker,
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_failed;
        if total == 0 {
            0.0
        } else {
            self.tasks_completed as f64 / total as f64
        }
    }
}
