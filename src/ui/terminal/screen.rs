// Wed Jan 15 2026 - Alex

use std::io::{self, Write};

pub struct Screen {
    width: u16,
    height: u16,
    buffer: Vec<Vec<char>>,
    alternate: bool,
}

impl Screen {
    pub fn new() -> Self {
        let (width, height) = terminal_size::terminal_size()
            .map(|(w, h)| (w.0, h.0))
            .unwrap_or((80, 24));

        Self {
            width,
            height,
            buffer: vec![vec![' '; width as usize]; height as usize],
            alternate: false,
        }
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
            self.buffer = vec![vec![' '; self.width as usize]; self.height as usize];
        }
    }

    pub fn clear(&mut self) -> io::Result<()> {
        print!("\x1B[2J\x1B[H");
        io::stdout().flush()?;
        self.buffer = vec![vec![' '; self.width as usize]; self.height as usize];
        Ok(())
    }

    pub fn clear_line(&self, row: u16) -> io::Result<()> {
        print!("\x1B[{};1H\x1B[2K", row);
        io::stdout().flush()
    }

    pub fn clear_to_end(&self) -> io::Result<()> {
        print!("\x1B[J");
        io::stdout().flush()
    }

    pub fn clear_to_start(&self) -> io::Result<()> {
        print!("\x1B[1J");
        io::stdout().flush()
    }

    pub fn enter_alternate(&mut self) -> io::Result<()> {
        if !self.alternate {
            print!("\x1B[?1049h");
            io::stdout().flush()?;
            self.alternate = true;
        }
        Ok(())
    }

    pub fn leave_alternate(&mut self) -> io::Result<()> {
        if self.alternate {
            print!("\x1B[?1049l");
            io::stdout().flush()?;
            self.alternate = false;
        }
        Ok(())
    }

    pub fn is_alternate(&self) -> bool {
        self.alternate
    }

    pub fn set_char(&mut self, row: u16, col: u16, c: char) {
        if row > 0 && row <= self.height && col > 0 && col <= self.width {
            self.buffer[(row - 1) as usize][(col - 1) as usize] = c;
        }
    }

    pub fn get_char(&self, row: u16, col: u16) -> Option<char> {
        if row > 0 && row <= self.height && col > 0 && col <= self.width {
            Some(self.buffer[(row - 1) as usize][(col - 1) as usize])
        } else {
            None
        }
    }

    pub fn write_at(&mut self, row: u16, col: u16, text: &str) -> io::Result<()> {
        print!("\x1B[{};{}H{}", row, col, text);
        io::stdout().flush()?;

        for (i, c) in text.chars().enumerate() {
            let c_col = col + i as u16;
            if c_col <= self.width {
                self.set_char(row, c_col, c);
            }
        }

        Ok(())
    }

    pub fn fill(&mut self, c: char) -> io::Result<()> {
        let line: String = std::iter::repeat(c).take(self.width as usize).collect();

        for row in 1..=self.height {
            self.write_at(row, 1, &line)?;
        }

        Ok(())
    }

    pub fn draw_box(&mut self, row: u16, col: u16, width: u16, height: u16) -> io::Result<()> {
        let top_left = '┌';
        let top_right = '┐';
        let bottom_left = '└';
        let bottom_right = '┘';
        let horizontal = '─';
        let vertical = '│';

        let top: String = format!("{}{}{}", 
            top_left,
            std::iter::repeat(horizontal).take((width - 2) as usize).collect::<String>(),
            top_right
        );

        let bottom: String = format!("{}{}{}", 
            bottom_left,
            std::iter::repeat(horizontal).take((width - 2) as usize).collect::<String>(),
            bottom_right
        );

        self.write_at(row, col, &top)?;

        for i in 1..height - 1 {
            self.write_at(row + i, col, &vertical.to_string())?;
            self.write_at(row + i, col + width - 1, &vertical.to_string())?;
        }

        self.write_at(row + height - 1, col, &bottom)?;

        Ok(())
    }

    pub fn draw_horizontal_line(&mut self, row: u16, col: u16, length: u16) -> io::Result<()> {
        let line: String = std::iter::repeat('─').take(length as usize).collect();
        self.write_at(row, col, &line)
    }

    pub fn draw_vertical_line(&mut self, row: u16, col: u16, length: u16) -> io::Result<()> {
        for i in 0..length {
            self.write_at(row + i, col, "│")?;
        }
        Ok(())
    }

    pub fn scroll_up(&self, n: u16) -> io::Result<()> {
        print!("\x1B[{}S", n);
        io::stdout().flush()
    }

    pub fn scroll_down(&self, n: u16) -> io::Result<()> {
        print!("\x1B[{}T", n);
        io::stdout().flush()
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if self.alternate {
            let _ = self.leave_alternate();
        }
    }
}
