// Tue Jan 13 2026 - Alex

use log::{Level, LevelFilter, Log, Metadata, Record};
use colored::*;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::fs::{File, OpenOptions};
use std::path::Path;

pub struct LoggingUtils;

impl LoggingUtils {
    pub fn init_logger(level: LevelFilter) {
        let logger = Box::new(ColoredLogger::new(level));
        log::set_boxed_logger(logger).ok();
        log::set_max_level(level);
    }

    pub fn init_logger_with_file(level: LevelFilter, file_path: &Path) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let logger = Box::new(FileLogger::new(level, file));
        log::set_boxed_logger(logger).ok();
        log::set_max_level(level);
        Ok(())
    }

    pub fn level_from_str(s: &str) -> LevelFilter {
        match s.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" | "warning" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        }
    }

    pub fn level_from_verbosity(verbosity: usize) -> LevelFilter {
        match verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

struct ColoredLogger {
    level: LevelFilter,
    use_color: AtomicBool,
}

impl ColoredLogger {
    fn new(level: LevelFilter) -> Self {
        Self {
            level,
            use_color: AtomicBool::new(true),
        }
    }

    fn format_level(&self, level: Level) -> ColoredString {
        match level {
            Level::Error => "ERROR".red().bold(),
            Level::Warn => "WARN ".yellow().bold(),
            Level::Info => "INFO ".green().bold(),
            Level::Debug => "DEBUG".blue().bold(),
            Level::Trace => "TRACE".magenta().bold(),
        }
    }
}

impl Log for ColoredLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = if self.use_color.load(Ordering::Relaxed) {
                self.format_level(record.level()).to_string()
            } else {
                format!("{:5}", record.level())
            };

            let target = if !record.target().is_empty() {
                format!("[{}]", record.target())
            } else {
                String::new()
            };

            eprintln!("{} {} {}", level_str, target.dimmed(), record.args());
        }
    }

    fn flush(&self) {}
}

struct FileLogger {
    level: LevelFilter,
    file: Mutex<File>,
}

impl FileLogger {
    fn new(level: LevelFilter, file: File) -> Self {
        Self {
            level,
            file: Mutex::new(file),
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = chrono_timestamp();

            let line = format!(
                "{} {:5} [{}] {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            );

            if let Ok(mut file) = self.file.lock() {
                let _ = file.write_all(line.as_bytes());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    format!("{}.{:03}", secs, millis)
}

pub struct ScopedTimer {
    name: String,
    start: std::time::Instant,
}

impl ScopedTimer {
    pub fn new(name: &str) -> Self {
        log::debug!("[TIMER] {} started", name);
        Self {
            name: name.to_string(),
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        log::debug!("[TIMER] {} took {:.2}ms", self.name, elapsed.as_secs_f64() * 1000.0);
    }
}

pub fn init_logger(verbose: bool) {
    let level = if verbose { LevelFilter::Debug } else { LevelFilter::Info };
    LoggingUtils::init_logger(level);
}

pub fn init_from_env() {
    env_logger::init();
}

pub fn scoped_timer(name: &str) -> ScopedTimer {
    ScopedTimer::new(name)
}

pub fn log_error(msg: &str) {
    log::error!("{}", msg);
}

pub fn log_warn(msg: &str) {
    log::warn!("{}", msg);
}

pub fn log_info(msg: &str) {
    log::info!("{}", msg);
}

pub fn log_debug(msg: &str) {
    log::debug!("{}", msg);
}

pub fn log_trace(msg: &str) {
    log::trace!("{}", msg);
}

pub struct ProgressLogger {
    name: String,
    total: usize,
    current: usize,
    last_percent: usize,
}

impl ProgressLogger {
    pub fn new(name: &str, total: usize) -> Self {
        log::info!("[{}] Starting (0/{})", name, total);
        Self {
            name: name.to_string(),
            total,
            current: 0,
            last_percent: 0,
        }
    }

    pub fn inc(&mut self) {
        self.current += 1;
        self.maybe_log();
    }

    pub fn set(&mut self, value: usize) {
        self.current = value;
        self.maybe_log();
    }

    fn maybe_log(&mut self) {
        if self.total == 0 {
            return;
        }

        let percent = (self.current * 100) / self.total;
        if percent > self.last_percent && percent % 10 == 0 {
            log::info!("[{}] Progress: {}% ({}/{})", self.name, percent, self.current, self.total);
            self.last_percent = percent;
        }
    }

    pub fn finish(&self) {
        log::info!("[{}] Completed ({}/{})", self.name, self.current, self.total);
    }
}

impl Drop for ProgressLogger {
    fn drop(&mut self) {
        if self.current > 0 {
            self.finish();
        }
    }
}
