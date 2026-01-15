// Tue Jan 13 2026 - Alex

use colored::*;
use std::cmp::max;

pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
    alignment: Vec<Alignment>,
    use_color: bool,
    use_unicode: bool,
    border_style: BorderStyle,
    header_style: HeaderStyle,
    max_width: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Ascii,
    Unicode,
    Rounded,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderStyle {
    None,
    Bold,
    Underline,
    Colored,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            column_widths: Vec::new(),
            alignment: Vec::new(),
            use_color: true,
            use_unicode: true,
            border_style: BorderStyle::Unicode,
            header_style: HeaderStyle::Bold,
            max_width: None,
        }
    }

    pub fn with_headers(mut self, headers: &[&str]) -> Self {
        self.headers = headers.iter().map(|s| s.to_string()).collect();
        self.column_widths = self.headers.iter().map(|h| h.len()).collect();
        self.alignment = vec![Alignment::Left; self.headers.len()];
        self
    }

    pub fn with_rows<T: std::fmt::Display>(mut self, rows: &[Vec<T>]) -> Self {
        for row in rows {
            let string_row: Vec<String> = row.iter().map(|c| c.to_string()).collect();

            for (i, cell) in string_row.iter().enumerate() {
                if i < self.column_widths.len() {
                    self.column_widths[i] = max(self.column_widths[i], cell.len());
                } else {
                    self.column_widths.push(cell.len());
                }
            }

            self.rows.push(string_row);
        }
        self
    }

    pub fn add_row<T: std::fmt::Display>(mut self, row: &[T]) -> Self {
        let string_row: Vec<String> = row.iter().map(|c| c.to_string()).collect();

        for (i, cell) in string_row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = max(self.column_widths[i], cell.len());
            } else {
                self.column_widths.push(cell.len());
            }
        }

        self.rows.push(string_row);
        self
    }

    pub fn with_alignment(mut self, column: usize, alignment: Alignment) -> Self {
        if column < self.alignment.len() {
            self.alignment[column] = alignment;
        }
        self
    }

    pub fn with_all_alignments(mut self, alignment: Alignment) -> Self {
        for a in &mut self.alignment {
            *a = alignment;
        }
        self
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn with_unicode(mut self, use_unicode: bool) -> Self {
        self.use_unicode = use_unicode;
        self
    }

    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    pub fn with_header_style(mut self, style: HeaderStyle) -> Self {
        self.header_style = style;
        self
    }

    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    fn get_border_chars(&self) -> BorderChars {
        match self.border_style {
            BorderStyle::None => BorderChars::none(),
            BorderStyle::Ascii => BorderChars::ascii(),
            BorderStyle::Unicode => BorderChars::unicode(),
            BorderStyle::Rounded => BorderChars::rounded(),
            BorderStyle::Double => BorderChars::double(),
        }
    }

    fn align_cell(&self, content: &str, width: usize, alignment: Alignment) -> String {
        match alignment {
            Alignment::Left => format!("{:<width$}", content, width = width),
            Alignment::Center => format!("{:^width$}", content, width = width),
            Alignment::Right => format!("{:>width$}", content, width = width),
        }
    }

    fn truncate_to_width(&self, content: &str, max_width: usize) -> String {
        if content.len() <= max_width {
            content.to_string()
        } else if max_width >= 3 {
            format!("{}...", &content[..max_width - 3])
        } else {
            content[..max_width].to_string()
        }
    }

    pub fn build(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut output = Vec::new();
        let chars = self.get_border_chars();

        let widths: Vec<usize> = if let Some(max_w) = self.max_width {
            let col_count = self.column_widths.len();
            let available = max_w.saturating_sub(col_count * 3 + 1);
            let per_col = available / col_count;
            self.column_widths.iter().map(|&w| w.min(per_col)).collect()
        } else {
            self.column_widths.clone()
        };

        if chars.has_border() {
            output.push(self.build_horizontal_line(&widths, &chars, LinePosition::Top));
        }

        if !self.headers.is_empty() {
            output.push(self.build_row(&self.headers, &widths, &chars, true));

            if chars.has_border() {
                output.push(self.build_horizontal_line(&widths, &chars, LinePosition::Middle));
            }
        }

        for (i, row) in self.rows.iter().enumerate() {
            output.push(self.build_row(row, &widths, &chars, false));

            if chars.has_border() && i < self.rows.len() - 1 && chars.middle_horizontal != ' ' {
            }
        }

        if chars.has_border() {
            output.push(self.build_horizontal_line(&widths, &chars, LinePosition::Bottom));
        }

        output.join("\n")
    }

    fn build_row(&self, cells: &[String], widths: &[usize], chars: &BorderChars, is_header: bool) -> String {
        let mut parts = Vec::new();

        if chars.has_border() {
            parts.push(chars.vertical.to_string());
        }

        for (i, cell) in cells.iter().enumerate() {
            let width = if i < widths.len() { widths[i] } else { cell.len() };
            let alignment = if i < self.alignment.len() { self.alignment[i] } else { Alignment::Left };

            let truncated = self.truncate_to_width(cell, width);
            let aligned = self.align_cell(&truncated, width, alignment);

            let formatted = if is_header && self.use_color {
                match self.header_style {
                    HeaderStyle::None => aligned,
                    HeaderStyle::Bold => aligned.bold().to_string(),
                    HeaderStyle::Underline => aligned.underline().to_string(),
                    HeaderStyle::Colored => aligned.cyan().bold().to_string(),
                }
            } else {
                aligned
            };

            parts.push(format!(" {} ", formatted));

            if chars.has_border() {
                parts.push(chars.vertical.to_string());
            }
        }

        parts.join("")
    }

    fn build_horizontal_line(&self, widths: &[usize], chars: &BorderChars, position: LinePosition) -> String {
        let (left, middle, right, horizontal) = match position {
            LinePosition::Top => (chars.top_left, chars.top_middle, chars.top_right, chars.horizontal),
            LinePosition::Middle => (chars.middle_left, chars.middle_middle, chars.middle_right, chars.middle_horizontal),
            LinePosition::Bottom => (chars.bottom_left, chars.bottom_middle, chars.bottom_right, chars.horizontal),
        };

        let mut parts = Vec::new();
        parts.push(left.to_string());

        for (i, &width) in widths.iter().enumerate() {
            parts.push(horizontal.to_string().repeat(width + 2));
            if i < widths.len() - 1 {
                parts.push(middle.to_string());
            }
        }

        parts.push(right.to_string());
        parts.join("")
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum LinePosition {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
struct BorderChars {
    horizontal: char,
    vertical: char,
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    top_middle: char,
    bottom_middle: char,
    middle_left: char,
    middle_right: char,
    middle_middle: char,
    middle_horizontal: char,
}

impl BorderChars {
    fn none() -> Self {
        Self {
            horizontal: ' ',
            vertical: ' ',
            top_left: ' ',
            top_right: ' ',
            bottom_left: ' ',
            bottom_right: ' ',
            top_middle: ' ',
            bottom_middle: ' ',
            middle_left: ' ',
            middle_right: ' ',
            middle_middle: ' ',
            middle_horizontal: ' ',
        }
    }

    fn ascii() -> Self {
        Self {
            horizontal: '-',
            vertical: '|',
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            top_middle: '+',
            bottom_middle: '+',
            middle_left: '+',
            middle_right: '+',
            middle_middle: '+',
            middle_horizontal: '-',
        }
    }

    fn unicode() -> Self {
        Self {
            horizontal: '─',
            vertical: '│',
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            top_middle: '┬',
            bottom_middle: '┴',
            middle_left: '├',
            middle_right: '┤',
            middle_middle: '┼',
            middle_horizontal: '─',
        }
    }

    fn rounded() -> Self {
        Self {
            horizontal: '─',
            vertical: '│',
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            top_middle: '┬',
            bottom_middle: '┴',
            middle_left: '├',
            middle_right: '┤',
            middle_middle: '┼',
            middle_horizontal: '─',
        }
    }

    fn double() -> Self {
        Self {
            horizontal: '═',
            vertical: '║',
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            top_middle: '╦',
            bottom_middle: '╩',
            middle_left: '╠',
            middle_right: '╣',
            middle_middle: '╬',
            middle_horizontal: '═',
        }
    }

    fn has_border(&self) -> bool {
        self.vertical != ' '
    }
}

pub fn create_table() -> TableBuilder {
    TableBuilder::new()
}

pub fn simple_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    TableBuilder::new()
        .with_headers(headers)
        .with_rows(rows)
        .build()
}

pub fn ascii_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    TableBuilder::new()
        .with_headers(headers)
        .with_rows(rows)
        .with_border_style(BorderStyle::Ascii)
        .build()
}

pub fn borderless_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    TableBuilder::new()
        .with_headers(headers)
        .with_rows(rows)
        .with_border_style(BorderStyle::None)
        .build()
}
