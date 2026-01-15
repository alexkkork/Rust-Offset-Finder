// Tue Jan 13 2026 - Alex

use crate::config::Config;
use crate::memory::MemoryReader;
use crate::engine::core::{Engine, EngineError, EngineState};
use crate::finders::result::FinderResults;
use crate::ui::progress::ProgressManager;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub struct EngineRunner {
    engine: Engine,
    progress_manager: Option<ProgressManager>,
    stop_flag: Arc<AtomicBool>,
}

impl EngineRunner {
    pub fn new(config: Config, reader: Arc<dyn MemoryReader>) -> Self {
        let engine = Engine::new(config, reader);

        Self {
            engine,
            progress_manager: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_progress(mut self, progress_manager: ProgressManager) -> Self {
        self.progress_manager = Some(progress_manager);
        self
    }

    pub fn run(&mut self) -> Result<FinderResults, EngineError> {
        self.stop_flag.store(false, Ordering::SeqCst);

        self.engine.initialize()?;

        if let Some(ref pm) = self.progress_manager {
            pm.set_status("Running offset generation...");
        }

        let results = self.engine.run()?;

        if let Some(ref pm) = self.progress_manager {
            pm.finish_all();
        }

        Ok(results)
    }

    pub fn run_async(mut self) -> EngineHandle {
        let stop_flag = self.stop_flag.clone();
        let (tx, rx) = std::sync::mpsc::channel();

        let handle = thread::spawn(move || {
            let result = self.run();
            let _ = tx.send(result);
        });

        EngineHandle {
            thread_handle: Some(handle),
            result_receiver: rx,
            stop_flag,
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.engine.state();
    }

    pub fn is_running(&self) -> bool {
        self.engine.state() == EngineState::Running
    }

    pub fn state(&self) -> EngineState {
        self.engine.state()
    }
}

pub struct EngineHandle {
    thread_handle: Option<thread::JoinHandle<()>>,
    result_receiver: std::sync::mpsc::Receiver<Result<FinderResults, EngineError>>,
    stop_flag: Arc<AtomicBool>,
}

impl EngineHandle {
    pub fn wait(mut self) -> Result<FinderResults, EngineError> {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        self.result_receiver.recv()
            .unwrap_or(Err(EngineError::InvalidState("No result received".to_string())))
    }

    pub fn try_get_result(&self) -> Option<Result<FinderResults, EngineError>> {
        self.result_receiver.try_recv().ok()
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn is_finished(&self) -> bool {
        self.result_receiver.try_recv().is_ok()
    }

    pub fn wait_timeout(mut self, timeout: Duration) -> Option<Result<FinderResults, EngineError>> {
        self.result_receiver.recv_timeout(timeout).ok()
    }
}

pub struct EngineRunnerBuilder {
    config: Option<Config>,
    reader: Option<Arc<dyn MemoryReader>>,
    progress_enabled: bool,
}

impl EngineRunnerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            reader: None,
            progress_enabled: false,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_reader(mut self, reader: Arc<dyn MemoryReader>) -> Self {
        self.reader = Some(reader);
        self
    }

    pub fn enable_progress(mut self, enabled: bool) -> Self {
        self.progress_enabled = enabled;
        self
    }

    pub fn build(self) -> Result<EngineRunner, EngineError> {
        let config = self.config
            .ok_or_else(|| EngineError::InvalidState("Config not set".to_string()))?;

        let reader = self.reader
            .ok_or_else(|| EngineError::InvalidState("Reader not set".to_string()))?;

        let mut runner = EngineRunner::new(config, reader);

        if self.progress_enabled {
            runner = runner.with_progress(ProgressManager::new());
        }

        Ok(runner)
    }
}

impl Default for EngineRunnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
