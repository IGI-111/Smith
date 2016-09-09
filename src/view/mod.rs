use state::{Selectable, Editable, Named};
use std::cmp;
use std::io::{stdout, Stdout, Write, Result};
use std::iter;
use termion::terminal_size;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, style, cursor, color};

pub struct View {
    stdout: MouseTerminal<RawTerminal<Stdout>>,
    message: Option<String>,
    is_prompt: bool,
    line_offset: u16,
}

const TAB_LENGTH: usize = 4;

impl View {
    pub fn new() -> Result<View> {
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        try!(write!(stdout, "{}{}", clear::All, cursor::Show));
        Ok(View {
            stdout: stdout,
            message: None,
            is_prompt: false,
            line_offset: 0,
        })
    }

    pub fn message(&mut self, message: &str) {
        self.is_prompt = false;
        self.message = Some(String::from(message));
    }

    pub fn prompt(&mut self, prompt: &str, message: &str) {
        self.is_prompt = true;
        let msg = String::from(prompt) + message;
        self.message = Some(msg);
    }

    pub fn quiet(&mut self) {
        self.is_prompt = false;
        self.message = None;
    }

    pub fn center_view(&mut self, line: usize) {
        self.line_offset = match line.checked_sub(self.lines_height() as usize / 2) {
            None => 0,
            Some(val) => val as u16,
        }
    }

    pub fn adjust_view(&mut self, line: usize) {
        let line = line as u16;
        if line < self.line_offset {
            self.line_offset = line;
        } else if line + 1 >= self.line_offset + self.lines_height() {
            self.line_offset = 1 + line - self.lines_height();
        }
    }

    pub fn scroll_view<T: Editable>(&mut self, offset: isize, content: &T) {
        self.line_offset =
            cmp::min(cmp::max((self.line_offset as isize) + offset, 0),
                     (content.line_count() as isize) - 1) as u16;

    }

    pub fn render<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Named + Selectable
    {
        try!(write!(self.stdout, "{}", clear::All));
        try!(self.paint_lines(content));
        try!(self.paint_status(content));
        try!(self.paint_message());
        try!(self.paint_cursor(content));
        try!(self.stdout.flush());
        Ok(())
    }

    pub fn translate_coordinates<T>(&self, content: &T, x: u16, y: u16) -> (usize, usize)
        where T: Editable
    {
        let line = y as isize + self.line_offset as isize - 1;
        let col = cmp::max(0,
                           x as isize - self.line_number_width(content.line_count()) as isize - 2);
        (line as usize, col as usize)
    }

    fn paint_message(&mut self) -> Result<()> {
        if let Some(ref message) = self.message {
            let y = self.lines_height() + 1;
            try!(write!(self.stdout, "{}{}", cursor::Goto(1, 1 + y as u16), message));
            try!(self.stdout.flush());
        }
        Ok(())
    }

    fn paint_cursor<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Selectable
    {
        // FIXME: don't print the cursor if off screen, though we should in the future for long
        // lines
        if (content.line() as u16) < self.line_offset ||
           content.line() as u16 >= self.line_offset + self.lines_height() ||
           content.col() as u16 >= self.lines_width(content.line_count()) ||
           content.sel().is_some() {
            try!(write!(self.stdout, "{}", cursor::Hide));
            return Ok(());
        }

        // in the case of a prompt, the cursor should be drawn in the message line
        let (x, y) = if self.is_prompt {
            (self.message.clone().unwrap().len() as u16, self.lines_height() + 1)
        } else {
            let (a, b) = self.cursor_pos(content);
            (a as u16, b as u16)
        };
        try!(write!(self.stdout,
                    "{}{}",
                    cursor::Show,
                    cursor::Goto(1 + x, 1 + y)));
        Ok(())
    }

    fn paint_status<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Named
    {
        let line = content.line();
        let column = content.col();
        let line_count = content.line_count();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();

        let (screen_width, _) = terminal_size().unwrap();
        let empty_line = (0..screen_width).map(|_| ' ').collect::<String>();
        let y = self.lines_height() as u16;

        try!(write!(self.stdout,
                    "{}{}{}{}{}{}",
                    color::Fg(color::White),
                    style::Invert,
                    cursor::Goto(1, 1 + y),
                    empty_line,
                    cursor::Goto(1, 1 + y),
                    content.name()));

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width - position_info.len() as u16;
        try!(write!(self.stdout,
                    "{}{}{}",
                    cursor::Goto(1 + x, 1 + y),
                    position_info,
                    style::Reset));
        Ok(())
    }

    fn paint_lines<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Selectable
    {
        let line_offset = self.line_offset;
        let lines_height = self.lines_height();
        let lines_width = self.lines_width(content.line_count());
        let line_count = content.line_count();

        let window_it = content.iter()
                .take(content.len() - 1) // the last endline should be invisible to the user
                .scan(('\n', 0, 1), |state, k| {
                    {
                        let (ref mut c, ref mut chars, ref mut lines) = *state;
                        if *c == '\n' {
                            *lines += 1;
                        }
                        *chars += 1;
                        *c = k;
                    }
                    Some(*state)
                })
                .skip_while(|&(_, _, lines)| lines <= 1 + line_offset)
                .take_while(|&(_, _, lines)| lines <= 1 + line_offset + lines_height);

        {
            let line_start = self.line_number_width(line_count) as u16 + 1;
            try!(write!(self.stdout,
                        "{}{}{}{}{}",
                        color::Fg(color::White),
                        cursor::Goto(1, 1),
                        line_offset + 1,
                        style::Reset,
                        cursor::Goto(1 + line_start, 1)));
        }
        let mut y = 1;
        let mut line_len = 0;
        for (c, chars, lines) in window_it {
            if c == '\n' {
                if line_len == 0 && content.in_sel(chars) {
                    try!(write!(self.stdout, "{} {}", style::Invert, style::Reset));
                }
                line_len = 0;
                let line_start = self.line_number_width(line_count) as u16 + 1;
                try!(write!(self.stdout,
                            "{}{}{}{}{}",
                            color::Fg(color::White),
                            cursor::Goto(1, 1 + y),
                            lines,
                            style::Reset,
                            cursor::Goto(1 + line_start, 1 + y)));
                y += 1;
            } else if line_len > lines_width - 1 {
                // don't print the character
                line_len += 1;
            } else {
                if content.in_sel(chars) {
                    try!(write!(self.stdout, "{}", style::Invert));
                }
                if c == '\t' {
                    // FIXME: we should probably use gotos instead of relying on character length
                    try!(write!(self.stdout,
                                "{}",
                                iter::repeat(" ").take(TAB_LENGTH).collect::<String>()));
                } else {
                    try!(write!(self.stdout, "{}", c));
                }
                try!(write!(self.stdout, "{}", style::Reset));
                line_len += 1;
            }
        }
        Ok(())
    }


    fn cursor_pos<T: Editable>(&self, content: &T) -> (usize, usize) {
        // TODO: column offsetting for long lines
        let line = content.line();
        let first_line = self.line_offset;
        let y = line - first_line as usize;
        // we can't trust the actual column because tabs have variable length
        let column = content.visual_col(TAB_LENGTH);
        ((self.line_number_width(content.line_count()) as usize + 1 + column), y)
    }

    fn line_number_width(&self, line_count: usize) -> u16 {
        let max_in_window = self.line_offset + self.lines_height() + 2;
        let max = cmp::min(max_in_window, line_count as u16);
        max.to_string().len() as u16
    }

    fn status_height(&self) -> u16 {
        2
    }

    pub fn lines_height(&self) -> u16 {
        let (_, screen_height) = terminal_size().unwrap();
        let incompressible = self.status_height();
        cmp::max(screen_height, incompressible) - incompressible
    }

    pub fn lines_width(&self, line_count: usize) -> u16 {
        let (screen_width, _) = terminal_size().unwrap();
        let incompressible = self.line_number_width(line_count) + 1;
        cmp::max(screen_width, incompressible) - incompressible
    }
}
