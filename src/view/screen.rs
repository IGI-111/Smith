use std;
use std::cell::RefCell;
use std::io;
use std::io::{BufWriter, Write};

use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use termion;
use termion::screen::AlternateScreen;
use termion::input::MouseTerminal;
use termion::color;
use termion::color::Color;
use termion::style;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Screen {
    out: RefCell<MouseTerminal<AlternateScreen<RawTerminal<BufWriter<io::Stdout>>>>>,
    buf: RefCell<Vec<Option<(Style, String)>>>,
    w: usize,
    h: usize,
    cursor_pos: (usize, usize),
}

impl Screen {
    pub fn new() -> Self {
        let (w, h) = termion::terminal_size().unwrap();
        let buf = std::iter::repeat(Some(("".into(), " ".into())))
            .take(w as usize * h as usize)
            .collect();
        Screen {
            out: RefCell::new(MouseTerminal::from(AlternateScreen::from(
                BufWriter::with_capacity(1 << 14, io::stdout())
                    .into_raw_mode()
                    .unwrap(),
            ))),
            buf: RefCell::new(buf),
            w: w as usize,
            h: h as usize,
            cursor_pos: (0, 0),
        }
    }

    pub fn clear<C>(&self, col: C)
    where
        C: Color + Clone,
    {
        for cell in self.buf.borrow_mut().iter_mut() {
            match *cell {
                Some((ref mut style, ref mut text)) => {
                    *style = format!("{}{}", color::Fg(col.clone()), color::Bg(col.clone()));
                    text.clear();
                    text.push_str(" ");
                }
                _ => {
                    *cell = Some((
                        format!(
                            "{}{}",
                            color::Fg(col.clone()),
                            color::Bg(col.clone())
                        ),
                        " ".into(),
                    ));
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.buf.borrow_mut().resize(
            w * h,
            Some((
                "".into(),
                " ".into(),
            )),
        );
    }

    pub fn present(&self) {
        let mut out = self.out.borrow_mut();
        let buf = self.buf.borrow();

        let mut last_style = "";
        write!(out, "{}", last_style).unwrap();

        // Write everything to the tmp_string first.
        for y in 0..self.h {
            let mut x = 0;
            write!(out, "{}", termion::cursor::Goto(1, y as u16 + 1)).unwrap();
            while x < self.w {
                if let Some((ref style, ref text)) = buf[y * self.w + x] {
                    if style != last_style {
                        write!(out, "{}{}", style::Reset, style).unwrap();
                        last_style = style;
                    }
                    write!(out, "{}", text).unwrap();
                    x += 1;
                } else {
                    x += 1;
                }
            }
        }

        let (cx, cy) = self.cursor_pos;
        write!( out, "{}", termion::cursor::Goto(1 + cx as u16, 1 + cy as u16)).unwrap();

        // Make sure everything is written out
        out.flush().unwrap();
    }
    pub fn draw(&self, x: usize, y: usize, text: &str) {
        self.draw_with_style(x, y, text, "".into());
    }

    pub fn draw_with_style(&self, x: usize, y: usize, text: &str, style: Style) {
        if y < self.h {
            let mut buf = self.buf.borrow_mut();
            let mut x = x;
            for g in UnicodeSegmentation::graphemes(text, true) {
                let width = UnicodeWidthStr::width(g);
                if width > 0 {
                    if x < self.w {
                        buf[y * self.w + x] = Some((style.clone(), g.into()));
                    }
                    x += 1;
                    for _ in 0..(width - 1) {
                        if x < self.w {
                            buf[y * self.w + x] = None;
                        }
                        x += 1;
                    }
                }
            }
        }
    }

    pub fn hide_cursor(&self) {
        write!(self.out.borrow_mut(), "{}", termion::cursor::Hide).unwrap();
    }

    pub fn show_cursor(&self) {
        write!(self.out.borrow_mut(), "{}", termion::cursor::Show).unwrap();
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_pos = (x, y);
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
        ).unwrap();
        self.show_cursor();
    }
}

pub type Style = String;
