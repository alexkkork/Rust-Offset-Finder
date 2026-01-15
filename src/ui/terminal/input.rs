// Wed Jan 15 2026 - Alex

use std::io::{self, Read, Write};

pub struct TerminalInput {
    buffer: Vec<u8>,
}

impl TerminalInput {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
        }
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        self.buffer.clear();
        io::stdin().read_line(&mut String::new())?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }

    pub fn read_char(&mut self) -> io::Result<char> {
        let mut buf = [0u8; 4];
        io::stdin().read_exact(&mut buf[..1])?;

        if buf[0] & 0x80 == 0 {
            return Ok(buf[0] as char);
        }

        let len = if buf[0] & 0xE0 == 0xC0 { 2 }
        else if buf[0] & 0xF0 == 0xE0 { 3 }
        else if buf[0] & 0xF8 == 0xF0 { 4 }
        else { 1 };

        if len > 1 {
            io::stdin().read_exact(&mut buf[1..len])?;
        }

        std::str::from_utf8(&buf[..len])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .chars()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty character"))
    }

    pub fn prompt(&mut self, message: &str) -> io::Result<String> {
        print!("{}", message);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }

    pub fn prompt_yes_no(&mut self, message: &str, default: bool) -> io::Result<bool> {
        let suffix = if default { "[Y/n]" } else { "[y/N]" };
        let input = self.prompt(&format!("{} {} ", message, suffix))?;

        let input = input.to_lowercase();
        if input.is_empty() {
            return Ok(default);
        }

        Ok(input == "y" || input == "yes")
    }

    pub fn prompt_choice(&mut self, message: &str, choices: &[&str]) -> io::Result<usize> {
        println!("{}", message);
        for (i, choice) in choices.iter().enumerate() {
            println!("  {}. {}", i + 1, choice);
        }

        loop {
            let input = self.prompt("Enter choice: ")?;

            if let Ok(n) = input.parse::<usize>() {
                if n >= 1 && n <= choices.len() {
                    return Ok(n - 1);
                }
            }

            println!("Invalid choice. Please enter a number between 1 and {}.", choices.len());
        }
    }

    pub fn prompt_password(&mut self, message: &str) -> io::Result<String> {
        print!("{}", message);
        io::stdout().flush()?;

        let password = rpassword::read_password()?;
        Ok(password)
    }
}

impl Default for TerminalInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    F(u8),
    Ctrl(char),
    Alt(char),
    Unknown,
}

impl Key {
    pub fn from_escape_sequence(bytes: &[u8]) -> Option<Key> {
        if bytes.is_empty() {
            return None;
        }

        if bytes[0] != 0x1B {
            return match bytes[0] {
                0x0D => Some(Key::Enter),
                0x7F | 0x08 => Some(Key::Backspace),
                0x09 => Some(Key::Tab),
                0x01..=0x1A => Some(Key::Ctrl((bytes[0] + 0x60) as char)),
                b if b < 0x80 => Some(Key::Char(b as char)),
                _ => None,
            };
        }

        if bytes.len() < 2 {
            return Some(Key::Escape);
        }

        if bytes[1] == b'[' {
            if bytes.len() < 3 {
                return None;
            }

            match bytes[2] {
                b'A' => return Some(Key::Up),
                b'B' => return Some(Key::Down),
                b'C' => return Some(Key::Right),
                b'D' => return Some(Key::Left),
                b'H' => return Some(Key::Home),
                b'F' => return Some(Key::End),
                b'2' if bytes.len() > 3 && bytes[3] == b'~' => return Some(Key::Insert),
                b'3' if bytes.len() > 3 && bytes[3] == b'~' => return Some(Key::Delete),
                b'5' if bytes.len() > 3 && bytes[3] == b'~' => return Some(Key::PageUp),
                b'6' if bytes.len() > 3 && bytes[3] == b'~' => return Some(Key::PageDown),
                _ => {}
            }
        }

        Some(Key::Unknown)
    }
}
