// Tue Jan 13 2026 - Alex

use crate::engine::task::Task;
use crate::engine::result::TaskResult;
use crate::finders::result::FinderResults;
use std::collections::BinaryHeap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use parking_lot::RwLock;

pub struct Worker {
    id: usize,
    task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
    result_sender: Sender<TaskResult>,
    running: Arc<RwLock<bool>>,
    thread_handle: Option<JoinHandle<()>>,
}

pub struct PrioritizedTask {
    pub task: Task,
    pub priority: i32,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl Worker {
    pub fn new(
        id: usize,
        task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
        result_sender: Sender<TaskResult>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            id,
            task_queue,
            result_sender,
            running,
            thread_handle: None,
        }
    }

    pub fn start(&mut self) {
        let id = self.id;
        let task_queue = self.task_queue.clone();
        let result_sender = self.result_sender.clone();
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            Worker::worker_loop(id, task_queue, result_sender, running);
        });

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    fn worker_loop(
        id: usize,
        task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
        result_sender: Sender<TaskResult>,
        running: Arc<RwLock<bool>>,
    ) {
        loop {
            if !*running.read() {
                break;
            }

            let task = {
                let mut queue = task_queue.lock().unwrap();
                queue.pop().map(|pt| pt.task)
            };

            match task {
                Some(task) => {
                    let start_time = Instant::now();
                    let result = Worker::execute_task(&task);
                    let duration = start_time.elapsed();

                    let _ = result_sender.send(result);
                }
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn execute_task(task: &Task) -> TaskResult {
        let mut results = FinderResults::new();

        match task.execute() {
            Ok(findings) => TaskResult::Success(findings),
            Err(e) => TaskResult::Error(e.to_string()),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn is_running(&self) -> bool {
        self.thread_handle.is_some()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
    result_sender: Sender<TaskResult>,
    running: Arc<RwLock<bool>>,
}

impl WorkerPool {
    pub fn new(
        size: usize,
        task_queue: Arc<Mutex<BinaryHeap<PrioritizedTask>>>,
        result_sender: Sender<TaskResult>,
    ) -> Self {
        let running = Arc::new(RwLock::new(false));
        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(
                i,
                task_queue.clone(),
                result_sender.clone(),
                running.clone(),
            ));
        }

        Self {
            workers,
            task_queue,
            result_sender,
            running,
        }
    }

    pub fn start(&mut self) {
        *self.running.write() = true;

        for worker in &mut self.workers {
            worker.start();
        }
    }

    pub fn stop(&mut self) {
        *self.running.write() = false;

        for worker in &mut self.workers {
            worker.stop();
        }
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }

    pub fn active_count(&self) -> usize {
        self.workers.iter().filter(|w| w.is_running()).count()
    }

    pub fn resize(&mut self, new_size: usize) {
        let current_size = self.workers.len();

        if new_size > current_size {
            for i in current_size..new_size {
                let mut worker = Worker::new(
                    i,
                    self.task_queue.clone(),
                    self.result_sender.clone(),
                    self.running.clone(),
                );

                if *self.running.read() {
                    worker.start();
                }

                self.workers.push(worker);
            }
        } else if new_size < current_size {
            while self.workers.len() > new_size {
                if let Some(mut worker) = self.workers.pop() {
                    worker.stop();
                }
            }
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.stop();
    }
}
