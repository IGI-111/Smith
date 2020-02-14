use ndarray::{Array, Array2};
use std;
use std::cell::RefCell;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::{BufWriter, Write};
use std::iter::FromIterator;
use syntect::highlighting::Style;
use termion;
use termion::color;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::style;

pub struct Screen {
    out: RefCell<MouseTerminal<AlternateScreen<RawTerminal<BufWriter<io::Stdout>>>>>,
    buf: RefCell<Array2<(Style, char)>>,
    cursor_pos: (usize, usize),
    cursor_visible: bool,
    default_style: Style,
}

impl Screen {
    pub fn with_default_style(default_style: Style) -> Self {
        let (w, h) = termion::terminal_size().unwrap();
        let buf =
            Array::from_iter(std::iter::repeat((default_style, ' ')).take(w as usize * h as usize));
        let buf = buf.into_shape((h as usize, w as usize)).unwrap();
        Screen {
            out: RefCell::new(MouseTerminal::from(AlternateScreen::from(
                BufWriter::with_capacity(1 << 14, io::stdout())
                    .into_raw_mode()
                    .unwrap(),
            ))),
            buf: RefCell::new(buf),
            cursor_pos: (0, 0),
            cursor_visible: true,
            default_style,
        }
    }

    pub fn clear(&self) {
        for cell in self.buf.borrow_mut().iter_mut() {
            *cell = (self.default_style, ' ');
        }
    }

    pub fn present(&self) {
        let mut out = self.out.borrow_mut();
        let buf = self.buf.borrow();

        write!(out, "{}", termion::cursor::Hide).unwrap();
        let mut last_style = self.default_style;
        write!(out, "{}", Self::escape_style(&last_style)).unwrap();

        let (h, w) = buf.dim();
        for y in 0..h {
            write!(out, "{}", termion::cursor::Goto(1, 1 + y as u16)).unwrap();
            for x in 0..w {
                let (style, ref text) = buf[[y, x]];
                if style != last_style {
                    write!(out, "{}{}", style::Reset, Self::escape_style(&style)).unwrap();
                    last_style = style;
                }
                write!(out, "{}", text).unwrap();
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
        let mut buf = self.buf.borrow_mut();
        let (h, w) = buf.dim();
        if y >= h {
            return;
        }
        let mut x = x;
        for (style, text) in ranges {
            for g in text.chars() {
                if x >= w {
                    break;
                }
                buf[[y, x]] = (style, g);
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
