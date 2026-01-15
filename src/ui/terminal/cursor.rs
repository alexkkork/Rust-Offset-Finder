// Wed Jan 15 2026 - Alex

use std::io::{self, Write};

pub struct Cursor {
    row: u16,
    col: u16,
    visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            row: 1,
            col: 1,
            visible: true,
        }
    }

    pub fn position(&self) -> (u16, u16) {
        (self.row, self.col)
    }

    pub fn set_position(&mut self, row: u16, col: u16) {
        self.row = row;
        self.col = col;
    }

    pub fn move_to(&mut self, row: u16, col: u16) -> io::Result<()> {
        self.row = row;
        self.col = col;
        print!("\x1B[{};{}H", row, col);
        io::stdout().flush()
    }

    pub fn move_up(&mut self, n: u16) -> io::Result<()> {
        self.row = self.row.saturating_sub(n);
        print!("\x1B[{}A", n);
        io::stdout().flush()
    }

    pub fn move_down(&mut self, n: u16) -> io::Result<()> {
        self.row = self.row.saturating_add(n);
        print!("\x1B[{}B", n);
        io::stdout().flush()
    }

    pub fn move_right(&mut self, n: u16) -> io::Result<()> {
        self.col = self.col.saturating_add(n);
        print!("\x1B[{}C", n);
        io::stdout().flush()
    }

    pub fn move_left(&mut self, n: u16) -> io::Result<()> {
        self.col = self.col.saturating_sub(n);
        print!("\x1B[{}D", n);
        io::stdout().flush()
    }

    pub fn move_to_column(&mut self, col: u16) -> io::Result<()> {
        self.col = col;
        print!("\x1B[{}G", col);
        io::stdout().flush()
    }

    pub fn move_to_start(&mut self) -> io::Result<()> {
        self.col = 1;
        print!("\r");
        io::stdout().flush()
    }

    pub fn save(&self) -> io::Result<()> {
        print!("\x1B[s");
        io::stdout().flush()
    }

    pub fn restore(&mut self) -> io::Result<()> {
        print!("\x1B[u");
        io::stdout().flush()
    }

    pub fn hide(&mut self) -> io::Result<()> {
        self.visible = false;
        print!("\x1B[?25l");
        io::stdout().flush()
    }

    pub fn show(&mut self) -> io::Result<()> {
        self.visible = true;
        print!("\x1B[?25h");
        io::stdout().flush()
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn blink(&mut self, enable: bool) -> io::Result<()> {
        if enable {
            print!("\x1B[?12h");
        } else {
            print!("\x1B[?12l");
        }
        io::stdout().flush()
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        if !self.visible {
            let _ = self.show();
        }
    }
}

pub struct CursorGuard {
    cursor: Cursor,
    saved_position: (u16, u16),
    was_visible: bool,
}

impl CursorGuard {
    pub fn new() -> io::Result<Self> {
        let mut cursor = Cursor::new();
        let saved_position = cursor.position();
        let was_visible = cursor.is_visible();
        cursor.save()?;
        Ok(Self {
            cursor,
            saved_position,
            was_visible,
        })
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        let _ = self.cursor.restore();
        if self.was_visible != self.cursor.is_visible() {
            if self.was_visible {
                let _ = self.cursor.show();
            } else {
                let _ = self.cursor.hide();
            }
        }
    }
}

impl Default for CursorGuard {
    fn default() -> Self {
        Self::new().expect("Failed to create cursor guard")
    }
}
