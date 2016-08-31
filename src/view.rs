use state::{Editable, Named};
use std::cmp;
use std::io::{stdout, Stdout, Write, Result};
use termion::terminal_size;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, style, cursor};

pub struct View {
    stdout: RawTerminal<Stdout>,
    message: Option<String>,
    is_prompt: bool,
}

impl View {
    pub fn new() -> Result<View> {
        let mut stdout = stdout().into_raw_mode().unwrap();
        try!(write!(stdout, "{}{}", clear::All, cursor::Show));
        Ok(View {
            stdout: stdout,
            message: None,
            is_prompt: false,
        })
    }

    pub fn message(&mut self, message: String) {
        self.is_prompt = false;
        self.message = Some(message);
    }

    pub fn prompt(&mut self, message: String) {
        self.is_prompt = true;
        self.message = Some(message);
    }

    pub fn quiet(&mut self) {
        self.is_prompt = false;
        self.message = None;
    }

    pub fn render<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Named
    {
        try!(write!(self.stdout, "{}", clear::All));
        try!(self.paint_lines(content));
        try!(self.paint_status(content));
        try!(self.paint_message());
        try!(self.paint_cursor(content));
        try!(self.stdout.flush());
        Ok(())
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
        where T: Editable
    {
        // in the case of a prompt, the cursor should be drawn in the message line
        let (x, y) = if self.is_prompt {
            (self.message.clone().unwrap().len(), self.lines_height() as usize + 1)
        } else {
             self.cursor_pos(content)
        };
        try!(write!(self.stdout, "{}", cursor::Goto(1 + x as u16, 1 + y as u16)));
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
                    "{}{}{}{}{}",
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

    fn paint_lines<T: Editable>(&mut self, content: &T) -> Result<()> {
        let line_offset = self.line_offset(content.line());
        let line_count = content.line_count();
        let window = content.as_rope()
            .line_iter()
            .take(line_count)
            .skip(line_offset as usize)
            .take(self.lines_height() as usize)
            .enumerate();

        let mut y = 0;
        for (relative_number, line) in window {
            let absolute_number = relative_number + line_offset as usize;

            try!(write!(self.stdout,
                        "{}{}",
                        cursor::Goto(1, 1 + y),
                        absolute_number + 1));

            let line_start = self.line_number_width(content.line(), line_count) as u16 + 1;
            try!(write!(self.stdout, "{}", cursor::Goto(1 + line_start, 1 + y)));
            for c in line.char_iter() {
                try!(write!(self.stdout, "{}", c));
            }
            y += 1;
        }
        Ok(())
    }

    fn cursor_pos<T: Editable>(&self, content: &T) -> (usize, usize) {
        // TODO: column offsetting for long lines
        let line = content.line();
        let column = content.col();
        let first_line = self.line_offset(line);
        let y = line - first_line as usize;
        ((self.line_number_width(line, content.line_count()) as usize + 1 + column), y)
    }

    fn line_number_width(&self, line: usize, line_count: usize) -> u16 {
        let max_in_window = self.line_offset(line) + self.lines_height() + 2;
        let max = cmp::min(max_in_window, line_count as u16);
        max.to_string().len() as u16
    }

    fn line_offset(&self, line: usize) -> u16 {
        match line.checked_sub(self.lines_height() as usize / 2) {
            None => 0,
            Some(val) => val as u16,
        }
    }

    fn status_height(&self) -> u16 {
        2
    }

    pub fn lines_height(&self) -> u16 {
        let (_, screen_height) = terminal_size().unwrap();
        cmp::max(screen_height, self.status_height()) - self.status_height()
    }
}
