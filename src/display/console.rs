use core::fmt;
use vga_framebuffer::freebsd_cp850::FONT_DATA;

use super::{WIDTH, HEIGHT};

const COLS: usize = WIDTH / FONT_WIDTH;
const LINES: usize = HEIGHT / FONT_HEIGHT;
const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

pub struct Console {
    pub buffer: [[char; COLS]; LINES],
    pub line: usize,
    pub col: usize,
}

impl Console {
    pub fn new() -> Self {
        Console {
            buffer: [[' '; COLS]; LINES],
            line: 0,
            col: 0,
        }
    }

    fn scroll(&mut self) {
        for line in 0..(LINES - 1) {
            self.buffer[line] = self.buffer[line + 1];
        }
        self.buffer[LINES - 1] = [' '; COLS];
        self.line -= 1;
    }
    
    pub fn add_char(&mut self, ch: char) {
        if self.col >= COLS {
            self.col = 0;
            self.line += 1;
        }
        while self.line >= LINES {
            self.scroll();
        }

        self.buffer[self.line][self.col] = ch;
        self.col += 1;
    }

    pub fn add_nl(&mut self) {
        // Cause scrolling in the next `add_char()` invokation
        self.col = COLS;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        let col = x / FONT_WIDTH;
        let line = y / FONT_HEIGHT;
        if col < COLS && line < LINES {
            let ch = self.buffer[line][col];
            let y1 = y % FONT_HEIGHT;
            let line = FONT_DATA[(ch as usize) * FONT_HEIGHT + y1];
            let x1 = x % FONT_WIDTH;
            (line & (0x80 >> x1)) != 0
        } else {
            false
        }
    }

    // TODO: fn render()
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if ch == '\n' {
                self.add_nl();
            } else {
                self.add_char(ch);
            }
        }

        Ok(())
    }
}
