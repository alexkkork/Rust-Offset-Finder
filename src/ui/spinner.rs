// Tue Jan 13 2026 - Alex

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use colored::*;

pub struct Spinner {
    message: String,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    style: SpinnerStyle,
    use_color: bool,
}

#[derive(Debug, Clone)]
pub struct SpinnerStyle {
    pub frames: Vec<&'static str>,
    pub interval_ms: u64,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            style: SpinnerStyle::dots(),
            use_color: true,
        }
    }

    pub fn with_style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let message = self.message.clone();
        let style = self.style.clone();
        let use_color = self.use_color;

        self.handle = Some(thread::spawn(move || {
            let mut frame_idx = 0;
            while running.load(Ordering::SeqCst) {
                let frame = style.frames[frame_idx % style.frames.len()];

                print!("\r");
                if use_color {
                    print!("{} {}", frame.cyan(), message);
                } else {
                    print!("{} {}", frame, message);
                }
                io::stdout().flush().unwrap();

                frame_idx += 1;
                thread::sleep(Duration::from_millis(style.interval_ms));
            }
        }));
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        print!("\r{}\r", " ".repeat(self.message.len() + 5));
        io::stdout().flush().unwrap();
    }

    pub fn stop_with_symbol(&mut self, symbol: &str) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        print!("\r");
        if self.use_color {
            println!("{} {}", symbol.green(), self.message);
        } else {
            println!("{} {}", symbol, self.message);
        }
    }

    pub fn stop_with_success(&mut self) {
        self.stop_with_symbol("✓");
    }

    pub fn stop_with_error(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        print!("\r");
        if self.use_color {
            println!("{} {}", "✗".red(), self.message.red());
        } else {
            println!("X {}", self.message);
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            self.stop();
        }
    }
}

impl SpinnerStyle {
    pub fn dots() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            interval_ms: 80,
        }
    }

    pub fn line() -> Self {
        Self {
            frames: vec!["-", "\\", "|", "/"],
            interval_ms: 100,
        }
    }

    pub fn arc() -> Self {
        Self {
            frames: vec!["◜", "◠", "◝", "◞", "◡", "◟"],
            interval_ms: 100,
        }
    }

    pub fn bounce() -> Self {
        Self {
            frames: vec!["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
            interval_ms: 100,
        }
    }

    pub fn arrow() -> Self {
        Self {
            frames: vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            interval_ms: 100,
        }
    }

    pub fn growing() -> Self {
        Self {
            frames: vec!["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃"],
            interval_ms: 80,
        }
    }

    pub fn clock() -> Self {
        Self {
            frames: vec!["🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛"],
            interval_ms: 100,
        }
    }

    pub fn simple() -> Self {
        Self {
            frames: vec![".", "..", "...", ""],
            interval_ms: 300,
        }
    }

    pub fn ascii() -> Self {
        Self {
            frames: vec!["|", "/", "-", "\\"],
            interval_ms: 100,
        }
    }

    pub fn custom(frames: Vec<&'static str>, interval_ms: u64) -> Self {
        Self { frames, interval_ms }
    }
}

pub struct SpinnerGuard {
    spinner: Spinner,
}

impl SpinnerGuard {
    pub fn new(message: &str) -> Self {
        let mut spinner = Spinner::new(message);
        spinner.start();
        Self { spinner }
    }

    pub fn with_style(message: &str, style: SpinnerStyle) -> Self {
        let mut spinner = Spinner::new(message).with_style(style);
        spinner.start();
        Self { spinner }
    }

    pub fn set_message(&mut self, message: &str) {
        self.spinner.set_message(message);
    }

    pub fn success(mut self) {
        self.spinner.stop_with_success();
    }

    pub fn error(mut self) {
        self.spinner.stop_with_error();
    }

    pub fn finish(mut self) {
        self.spinner.stop();
    }

    pub fn finish_with(mut self, symbol: &str) {
        self.spinner.stop_with_symbol(symbol);
    }
}

impl Drop for SpinnerGuard {
    fn drop(&mut self) {
        if self.spinner.is_running() {
            self.spinner.stop();
        }
    }
}

pub fn with_spinner<F, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let guard = SpinnerGuard::new(message);
    let result = f();
    guard.success();
    result
}

pub fn with_spinner_result<F, T, E>(message: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    let guard = SpinnerGuard::new(message);
    let result = f();
    match &result {
        Ok(_) => guard.success(),
        Err(_) => guard.error(),
    }
    result
}

pub fn create_spinner(message: &str) -> Spinner {
    Spinner::new(message)
}

pub fn create_spinner_guard(message: &str) -> SpinnerGuard {
    SpinnerGuard::new(message)
}
