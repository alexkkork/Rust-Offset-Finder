// Wed Jan 15 2026 - Alex

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct ErrorReporter {
    log_file: Option<Arc<Mutex<File>>>,
    error_count: Arc<Mutex<usize>>,
    max_errors: usize,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            log_file: None,
            error_count: Arc::new(Mutex::new(0)),
            max_errors: 1000,
        }
    }

    pub fn with_log_file(mut self, path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        self.log_file = Some(Arc::new(Mutex::new(file)));
        Ok(self)
    }

    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }

    pub fn report<E: Error + ?Sized>(&self, error: &E) {
        let mut count = self.error_count.lock().unwrap();
        *count += 1;

        if *count > self.max_errors {
            return;
        }

        if let Some(ref log_file) = self.log_file {
            if let Ok(mut file) = log_file.lock() {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let _ = writeln!(file, "[{}] Error: {}", timestamp, error);

                let mut source = error.source();
                while let Some(cause) = source {
                    let _ = writeln!(file, "  Caused by: {}", cause);
                    source = cause.source();
                }

                let _ = writeln!(file, "");
            }
        }
    }

    pub fn error_count(&self) -> usize {
        *self.error_count.lock().unwrap()
    }

    pub fn reset_count(&self) {
        let mut count = self.error_count.lock().unwrap();
        *count = 0;
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    pub fn summary(&self) -> String {
        let count = self.error_count();
        if count == 0 {
            "No errors reported".to_string()
        } else if count == 1 {
            "1 error reported".to_string()
        } else if count > self.max_errors {
            format!("{} errors reported (truncated at {})", count, self.max_errors)
        } else {
            format!("{} errors reported", count)
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ErrorReporter {
    fn clone(&self) -> Self {
        Self {
            log_file: self.log_file.clone(),
            error_count: self.error_count.clone(),
            max_errors: self.max_errors,
        }
    }
}
