// Tue Jan 13 2026 - Alex

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use parking_lot::{Mutex, Condvar};
use rayon::prelude::*;

pub struct DiscoveryScheduler {
    thread_count: usize,
    task_queue: Arc<Mutex<VecDeque<ScheduledTask>>>,
    running: Arc<AtomicBool>,
    pending_count: Arc<AtomicUsize>,
    completed_count: Arc<AtomicUsize>,
    condvar: Arc<Condvar>,
}

impl DiscoveryScheduler {
    pub fn new(thread_count: usize) -> Self {
        let thread_count = thread_count.max(1);
        Self {
            thread_count,
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(AtomicBool::new(false)),
            pending_count: Arc::new(AtomicUsize::new(0)),
            completed_count: Arc::new(AtomicUsize::new(0)),
            condvar: Arc::new(Condvar::new()),
        }
    }

    pub fn schedule(&self, task: ScheduledTask) {
        let mut queue = self.task_queue.lock();
        queue.push_back(task);
        self.pending_count.fetch_add(1, Ordering::SeqCst);
        self.condvar.notify_one();
    }

    pub fn schedule_batch(&self, tasks: Vec<ScheduledTask>) {
        let mut queue = self.task_queue.lock();
        let count = tasks.len();
        for task in tasks {
            queue.push_back(task);
        }
        self.pending_count.fetch_add(count, Ordering::SeqCst);
        self.condvar.notify_all();
    }

    pub fn start(&self) -> Vec<thread::JoinHandle<()>> {
        self.running.store(true, Ordering::SeqCst);
        let mut handles = Vec::with_capacity(self.thread_count);

        for _ in 0..self.thread_count {
            let queue = self.task_queue.clone();
            let running = self.running.clone();
            let pending = self.pending_count.clone();
            let completed = self.completed_count.clone();
            let condvar = self.condvar.clone();

            let handle = thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    let task = {
                        let mut queue_lock = queue.lock();
                        if queue_lock.is_empty() {
                            condvar.wait(&mut queue_lock);
                            queue_lock.pop_front()
                        } else {
                            queue_lock.pop_front()
                        }
                    };

                    if let Some(task) = task {
                        task.execute();
                        pending.fetch_sub(1, Ordering::SeqCst);
                        completed.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            handles.push(handle);
        }

        handles
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.condvar.notify_all();
    }

    pub fn wait_completion(&self) {
        while self.pending_count.load(Ordering::SeqCst) > 0 {
            thread::yield_now();
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending_count.load(Ordering::SeqCst)
    }

    pub fn completed_count(&self) -> usize {
        self.completed_count.load(Ordering::SeqCst)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn clear(&self) {
        let mut queue = self.task_queue.lock();
        let count = queue.len();
        queue.clear();
        self.pending_count.fetch_sub(count, Ordering::SeqCst);
    }
}

pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub priority: TaskPriority,
    pub task_fn: Box<dyn FnOnce() + Send + 'static>,
}

impl ScheduledTask {
    pub fn new<F>(id: u64, name: String, priority: TaskPriority, task_fn: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            id,
            name,
            priority,
            task_fn: Box::new(task_fn),
        }
    }

    pub fn execute(self) {
        (self.task_fn)();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

pub struct PriorityScheduler {
    queues: [Arc<Mutex<VecDeque<ScheduledTask>>>; 4],
    running: Arc<AtomicBool>,
    thread_count: usize,
}

impl PriorityScheduler {
    pub fn new(thread_count: usize) -> Self {
        Self {
            queues: [
                Arc::new(Mutex::new(VecDeque::new())),
                Arc::new(Mutex::new(VecDeque::new())),
                Arc::new(Mutex::new(VecDeque::new())),
                Arc::new(Mutex::new(VecDeque::new())),
            ],
            running: Arc::new(AtomicBool::new(false)),
            thread_count: thread_count.max(1),
        }
    }

    pub fn schedule(&self, task: ScheduledTask) {
        let idx = task.priority as usize;
        let mut queue = self.queues[idx].lock();
        queue.push_back(task);
    }

    pub fn get_next_task(&self) -> Option<ScheduledTask> {
        for queue in self.queues.iter().rev() {
            let mut q = queue.lock();
            if let Some(task) = q.pop_front() {
                return Some(task);
            }
        }
        None
    }

    pub fn start(&self) -> Vec<thread::JoinHandle<()>> {
        self.running.store(true, Ordering::SeqCst);
        let mut handles = Vec::with_capacity(self.thread_count);

        for _ in 0..self.thread_count {
            let queues = self.queues.clone();
            let running = self.running.clone();

            let handle = thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    let task = {
                        for queue in queues.iter().rev() {
                            let mut q = queue.lock();
                            if let Some(task) = q.pop_front() {
                                drop(q);
                                break;
                            }
                        }
                        None::<ScheduledTask>
                    };

                    if let Some(task) = task {
                        task.execute();
                    } else {
                        thread::yield_now();
                    }
                }
            });
            handles.push(handle);
        }

        handles
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn total_pending(&self) -> usize {
        self.queues.iter()
            .map(|q| q.lock().len())
            .sum()
    }
}

pub struct BatchExecutor {
    thread_count: usize,
}

impl BatchExecutor {
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count: thread_count.max(1),
        }
    }

    pub fn execute_parallel<T, F>(&self, items: Vec<T>, f: F) -> Vec<T::Output>
    where
        T: Send + Sync,
        T::Output: Send,
        F: Fn(&T) -> T::Output + Send + Sync,
        T: ParallelTask,
    {
        items.par_iter()
            .map(|item| f(item))
            .collect()
    }

    pub fn execute_sequential<T, F, R>(&self, items: Vec<T>, f: F) -> Vec<R>
    where
        F: Fn(T) -> R,
    {
        items.into_iter()
            .map(|item| f(item))
            .collect()
    }
}

pub trait ParallelTask {
    type Output;
    fn execute(&self) -> Self::Output;
}
