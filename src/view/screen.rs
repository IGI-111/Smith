use ndarray::{Array, Array2};
use std::cell::RefCell;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::{BufWriter, Write};
use syntect::highlighting::Style;
use termion::color;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

pub struct Screen {
    out: RefCell<RawTerminal<MouseTerminal<AlternateScreen<BufWriter<io::Stdout>>>>>,
    write_buf: RefCell<Array2<(Style, char)>>,
    read_buf: RefCell<Array2<(Style, char)>>,
    cursor_pos: (usize, usize),
    cursor_visible: bool,
    default_style: Style,
}

impl Screen {
    pub fn with_default_style(default_style: Style) -> Self {
        let (w, h) = termion::terminal_size().unwrap();
        let write_buf: Array<_, _> = std::iter::repeat((default_style, ' '))
            .take(w as usize * h as usize)
            .collect();
        let write_buf = write_buf.into_shape((h as usize, w as usize)).unwrap();
        let read_buf: Array<_, _> =
            (std::iter::repeat((default_style, 'X')).take(w as usize * h as usize)).collect();
        let read_buf = read_buf.into_shape((h as usize, w as usize)).unwrap();
        let out = RefCell::new(
            MouseTerminal::from(AlternateScreen::from(BufWriter::with_capacity(
                1 << 14,
                io::stdout(),
            )))
            .into_raw_mode()
            .unwrap(),
        );
        Screen {
            out,
            read_buf: RefCell::new(read_buf),
            write_buf: RefCell::new(write_buf),
            cursor_pos: (0, 0),
            cursor_visible: true,
            default_style,
        }
    }

    pub fn clear(&self) {
        for cell in self.write_buf.borrow_mut().iter_mut() {
            *cell = (self.default_style, ' ');
        }
    }

    pub fn present(&self) {
        let mut out = self.out.borrow_mut();
        let mut read_buf = self.read_buf.borrow_mut();
        let write_buf = self.write_buf.borrow();

        write!(out, "{}", termion::cursor::Hide).unwrap();
        let mut last_style = self.default_style;
        write!(out, "{}", Self::escape_style(&last_style)).unwrap();

        let (h, w) = write_buf.dim();
        for y in 0..h {
            for x in 0..w {
                if write_buf[[y, x]] != read_buf[[y, x]] {
                    read_buf[[y, x]] = write_buf[[y, x]];

                    let (style, ref text) = write_buf[[y, x]];
                    if style != last_style {
                        write!(out, "{}", Self::escape_style(&style)).unwrap();
                        last_style = style;
                    }
                    write!(
                        out,
                        "{}{}",
                        termion::cursor::Goto(1 + x as u16, 1 + y as u16),
                        text
                    )
                    .unwrap();
                }
            }
        }

        if self.cursor_visible {
            let (cx, cy) = self.cursor_pos;
            write!(
                out,
                "{}{}",
                termion::cursor::Goto(1 + cx as u16, 1 + cy as u16),
                termion::cursor::Show,
            )
            .unwrap();
        }

        // Make sure everything is written out
        out.flush().unwrap();
    }
    pub fn draw(&self, x: usize, y: usize, text: &str) {
        self.draw_with_style(x, y, self.default_style, text);
    }

    pub fn draw_with_style(&self, x: usize, y: usize, style: Style, text: &str) {
        self.draw_ranges(x, y, vec![(style, text)]);
    }

    pub fn draw_ranges(&self, x: usize, y: usize, ranges: Vec<(Style, &str)>) {
        let mut write_buf = self.write_buf.borrow_mut();
        let (h, w) = write_buf.dim();
        if y >= h {
            return;
        }
        let mut x = x;
        for (style, text) in ranges {
            for g in text.chars() {
                if x >= w {
                    break;
                }
                write_buf[[y, x]] = (style, g);
                x += 1;
            }
        }
    }

    pub fn hide_cursor(&mut self) {
        self.cursor_visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.cursor_visible = true;
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_pos = (x, y);
    }

    fn escape_style(style: &Style) -> String {
        let mut s = String::new();
        write!(
            s,
            "\x1b[48;2;{};{};{}m",
            style.background.r, style.background.g, style.background.b
        )
        .unwrap();
        write!(
            s,
            "\x1b[38;2;{};{};{}m",
            style.foreground.r, style.foreground.g, style.foreground.b
        )
        .unwrap();
        s
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        write!(
            self.out.borrow_mut(),
            "{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            termion::clear::All,
        )
        .unwrap();
        self.show_cursor();
    }
}
