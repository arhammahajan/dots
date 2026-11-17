use std::fmt::Write as _;

use unicode_width::UnicodeWidthChar;

use crate::theme::Style;
use crate::util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    ch: char,
    style: Style,
    continuation: bool,
}

pub struct Screen {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Screen {
    pub fn new(width: usize, height: usize, style: Style) -> Self {
        Screen {
            width,
            height,
            cells: vec![
                Cell {
                    ch: ' ',
                    style,
                    continuation: false,
                };
                width * height
            ],
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn fill(&mut self, style: Style) {
        self.cells.fill(Cell {
            ch: ' ',
            style,
            continuation: false,
        });
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, ch: char, style: Style) {
        for row in y..y.saturating_add(h).min(self.height) {
            for col in x..x.saturating_add(w).min(self.width) {
                let index = row * self.width + col;
                self.cells[index] = Cell {
                    ch,
                    style,
                    continuation: false,
                };
            }
        }
    }

    pub fn put(&mut self, x: usize, y: usize, text: &str, style: Style) {
        if y >= self.height {
            return;
        }
        let mut col = x;
        for ch in text.chars() {
            if col >= self.width {
                break;
            }
            if ch == '\n' {
                break;
            }
            // Pane titles and attention messages come from arbitrary programs.
            // Never let a control byte reach the host terminal through our UI.
            let ch = if ch.is_control() { ' ' } else { ch };
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if ch_width == 0 {
                continue;
            }
            if col + ch_width > self.width {
                break;
            }
            let index = y * self.width + col;
            self.cells[index] = Cell {
                ch,
                style,
                continuation: false,
            };
            for extra in 1..ch_width {
                let extra_index = y * self.width + col + extra;
                self.cells[extra_index] = Cell {
                    ch: ' ',
                    style,
                    continuation: true,
                };
            }
            col += ch_width;
        }
    }

    pub fn put_right(&mut self, right: usize, y: usize, text: &str, style: Style) {
        let text = util::truncate(text, right);
        let text_width = util::width(&text);
        self.put(right.saturating_sub(text_width), y, &text, style);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_box(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        border: Style,
        fill: Style,
        title: Option<(&str, Style)>,
        right_title: Option<(&str, Style)>,
    ) {
        if w < 2 || h < 2 {
            return;
        }
        self.fill_rect(x, y, w, h, ' ', fill);
        let right = x + w - 1;
        let bottom = y + h - 1;
        self.put(x, y, "╭", border);
        self.put(right, y, "╮", border);
        self.put(x, bottom, "╰", border);
        self.put(right, bottom, "╯", border);
        for col in x + 1..right {
            self.put(col, y, "─", border);
            self.put(col, bottom, "─", border);
        }
        for row in y + 1..bottom {
            self.put(x, row, "│", border);
            self.put(right, row, "│", border);
        }
        if let Some((title, style)) = title {
            let room = w.saturating_sub(4);
            let title = util::truncate(title, room);
            self.put(x + 2, y, &title, style);
        }
        if let Some((title, style)) = right_title {
            let room = w.saturating_sub(4);
            let title = util::truncate(title, room);
            let title_width = util::width(&title);
            self.put(right.saturating_sub(title_width + 1), y, &title, style);
        }
    }

    #[cfg(test)]
    pub fn to_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.height);
        for row in 0..self.height {
            let mut line = String::with_capacity(self.width);
            for col in 0..self.width {
                let cell = self.cells[row * self.width + col];
                if cell.continuation {
                    continue;
                }
                line.push(cell.ch);
            }
            lines.push(line);
        }
        lines
    }

    pub fn to_ansi(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height * 2);
        // Exact-width rows are written to the last column. Disable autowrap so
        // a wide glyph or a terminal that renders a cell wider than expected
        // cannot push the row into the next pane.
        out.push_str("\x1b[0m\x1b[?7l");
        for row in 0..self.height {
            let _ = write!(out, "\x1b[{};1H", row + 1);
            let mut current: Option<Style> = None;
            for col in 0..self.width {
                let cell = self.cells[row * self.width + col];
                if cell.continuation {
                    continue;
                }
                if current != Some(cell.style) {
                    write_style(&mut out, cell.style);
                    current = Some(cell.style);
                }
                out.push(cell.ch);
            }
        }
        out.push_str("\x1b[0m");
        out
    }
}

fn write_style(out: &mut String, style: Style) {
    out.push_str("\x1b[0m");
    let _ = write!(
        out,
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
        style.fg.0, style.fg.1, style.fg.2, style.bg.0, style.bg.1, style.bg.2
    );
    if style.bold {
        out.push_str("\x1b[1m");
    }
    if style.dim {
        out.push_str("\x1b[2m");
    }
    if style.italic {
        out.push_str("\x1b[3m");
    }
    if style.underline {
        out.push_str("\x1b[4m");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{BASE, BLUE, RED, TEXT};

    fn style() -> Style {
        Style::new(TEXT, BASE)
    }

    #[test]
    fn writes_and_clears_cells() {
        let mut screen = Screen::new(10, 3, style());
        screen.put(1, 1, "hello", Style::new(RED, BASE));
        assert_eq!(screen.to_lines()[1], " hello    ");
        screen.fill_rect(0, 0, 10, 3, ' ', style());
        assert_eq!(screen.to_lines()[1], "          ");
    }

    #[test]
    fn clips_text_at_the_edge() {
        let mut screen = Screen::new(5, 1, style());
        screen.put(3, 0, "abcdef", style());
        assert_eq!(screen.to_lines()[0], "   ab");
    }

    #[test]
    fn wide_characters_take_two_cells() {
        let mut screen = Screen::new(6, 1, style());
        screen.put(0, 0, "日本語", style());
        assert_eq!(screen.to_lines()[0], "日本語");
        assert_eq!(util::width(&screen.to_lines()[0]), 6);
    }

    #[test]
    fn a_wide_character_that_does_not_fit_is_not_split() {
        let mut screen = Screen::new(4, 1, style());
        screen.put(3, 0, "日", style());
        assert_eq!(screen.to_lines()[0], "    ");
    }

    #[test]
    fn right_alignment_never_goes_past_the_margin() {
        let mut screen = Screen::new(8, 1, style());
        screen.put_right(7, 0, "abc", style());
        assert_eq!(screen.to_lines()[0], "    abc ");
        screen.put_right(2, 0, "abcdef", style());
        assert_eq!(screen.to_lines()[0], "a…  abc ");
    }

    #[test]
    fn ansi_output_keeps_truecolor_and_does_not_emit_wide_char_padding() {
        let mut screen = Screen::new(4, 1, style());
        screen.put(0, 0, "日a", Style::new(BLUE, BASE));
        let ansi = screen.to_ansi();
        assert!(ansi.contains("\x1b[38;2;137;180;250m"));
        assert!(ansi.contains('日'));
        assert!(!ansi.contains("日 "));
    }

    #[test]
    fn control_characters_from_pane_text_are_neutralized() {
        let mut screen = Screen::new(8, 1, style());
        screen.put(0, 0, "a\x1b[31mb", style());
        assert_eq!(screen.to_lines()[0], "a [31mb ");
    }

    #[test]
    fn boxes_overlay_their_contents() {
        let mut screen = Screen::new(12, 5, style());
        screen.draw_box(
            0,
            0,
            12,
            5,
            Style::new(BLUE, BASE),
            style(),
            Some((" hi ", Style::new(TEXT, BASE))),
            None,
        );
        let lines = screen.to_lines();
        assert!(lines[0].starts_with("╭─ hi "));
        assert!(lines[0].ends_with('╮'));
        assert!(lines[4].starts_with('╰'));
        assert!(lines[4].ends_with('╯'));
    }
}
