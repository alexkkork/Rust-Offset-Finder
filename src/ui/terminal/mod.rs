// Wed Jan 15 2026 - Alex

pub mod color;
pub mod cursor;
pub mod input;
pub mod screen;

pub use color::TerminalColor;
pub use cursor::Cursor;
pub use input::TerminalInput;
pub use screen::Screen;

use std::io::{self, Write};

pub struct Terminal {
    stdout: io::Stdout,
    is_tty: bool,
    width: u16,
    height: u16,
}

impl Terminal {
    pub fn new() -> Self {
        let (width, height) = terminal_size::terminal_size()
            .map(|(w, h)| (w.0, h.0))
            .unwrap_or((80, 24));

        Self {
            stdout: io::stdout(),
            is_tty: atty::is(atty::Stream::Stdout),
            width,
            height,
        }
    }

    pub fn is_tty(&self) -> bool {
        self.is_tty
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn refresh_size(&mut self) {
        if let Some((w, h)) = terminal_size::terminal_size() {
            self.width = w.0;
            self.height = h.0;
        }
    }

    pub fn write(&mut self, text: &str) -> io::Result<()> {
        write!(self.stdout, "{}", text)?;
        self.stdout.flush()
    }

    pub fn writeln(&mut self, text: &str) -> io::Result<()> {
        writeln!(self.stdout, "{}", text)?;
        self.stdout.flush()
    }

    pub fn clear_line(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[2K\r")?;
        self.stdout.flush()
    }

    pub fn clear_screen(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[2J\x1B[H")?;
        self.stdout.flush()
    }

    pub fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        write!(self.stdout, "\x1B[{};{}H", row, col)?;
        self.stdout.flush()
    }

    pub fn move_up(&mut self, n: u16) -> io::Result<()> {
        write!(self.stdout, "\x1B[{}A", n)?;
        self.stdout.flush()
    }

    pub fn move_down(&mut self, n: u16) -> io::Result<()> {
        write!(self.stdout, "\x1B[{}B", n)?;
        self.stdout.flush()
    }

    pub fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[?25l")?;
        self.stdout.flush()
    }

    pub fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[?25h")?;
        self.stdout.flush()
    }

    pub fn save_cursor(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[s")?;
        self.stdout.flush()
    }

    pub fn restore_cursor(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1B[u")?;
        self.stdout.flush()
    }

    pub fn set_title(&mut self, title: &str) -> io::Result<()> {
        write!(self.stdout, "\x1B]0;{}\x07", title)?;
        self.stdout.flush()
    }

    pub fn bell(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x07")?;
        self.stdout.flush()
    }

    pub fn print_info(&self, message: &str) -> io::Result<()> {
        writeln!(io::stdout(), "ℹ {}", message)?;
        io::stdout().flush()
    }

    pub fn print_success(&self, message: &str) -> io::Result<()> {
        writeln!(io::stdout(), "✓ {}", message)?;
        io::stdout().flush()
    }

    pub fn print_warning(&self, message: &str) -> io::Result<()> {
        writeln!(io::stdout(), "⚠ {}", message)?;
        io::stdout().flush()
    }

    pub fn print_error(&self, message: &str) -> io::Result<()> {
        writeln!(io::stdout(), "✗ {}", message)?;
        io::stdout().flush()
    }

    pub fn print_debug(&self, message: &str) -> io::Result<()> {
        writeln!(io::stdout(), "🔍 {}", message)?;
        io::stdout().flush()
    }

    pub fn print_section(&self, title: &str) -> io::Result<()> {
        writeln!(io::stdout(), "\n=== {} ===", title)?;
        io::stdout().flush()
    }

    pub fn confirm(&self, message: &str) -> bool {
        use std::io::{self, BufRead};
        print!("{} [y/N]: ", message);
        let _ = io::stdout().flush();
        let stdin = io::stdin();
        let mut line = String::new();
        if let Ok(_) = stdin.lock().read_line(&mut line) {
            line.trim().to_lowercase() == "y"
        } else {
            false
        }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}
