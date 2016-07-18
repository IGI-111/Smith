use state::{Position, Editable, Named};
use std::cmp;
use std::io::{stdout, Stdout, Write, Result};
use termion::{Style, TermWrite, IntoRawMode, RawTerminal, terminal_size};

pub struct View {
    stdout: RawTerminal<Stdout>,
    message: Option<String>,
    prompt: Option<String>,
}

impl View {
    pub fn new() -> Result<View> {
        let mut stdout = stdout().into_raw_mode().unwrap();
        try!(stdout.clear());
        try!(stdout.show_cursor());
        Ok(View {
            stdout: stdout,
            message: None,
            prompt: None,
        })
    }

    pub fn message(&mut self, message: String) {
        self.message = Some(message);
    }

    pub fn prompt(&mut self, prompt: String, message: String) {
        self.message = Some(message);
        self.prompt = Some(prompt);
    }

    pub fn quiet(&mut self) {
        self.message = None;
        self.prompt = None;
    }

    pub fn render<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Named
    {
        try!(self.stdout.clear());
        try!(self.paint_lines(content));
        try!(self.paint_status(content));
        try!(self.paint_message());
        try!(self.paint_cursor(content));
        try!(self.stdout.flush());
        Ok(())
    }

    fn paint_message(&mut self) -> Result<()> {
        match self.message {
            Some(ref message) => {
                let y = self.lines_height() + 1;
                try!(self.stdout.goto(0, y as u16));
                try!(write!(self.stdout, "{}", message));
                try!(self.stdout.flush());
            }
            None => {}
        }
        Ok(())
    }

    fn paint_cursor<T>(&mut self, content: &T) -> Result<()>
        where T: Editable
    {
        //TODO: if there is a prompt, draw the cursor at the right position
        let (x, y) = self.cursor_pos(content);
        try!(self.stdout.goto(x as u16, y as u16));
        Ok(())
    }

    fn paint_status<T>(&mut self, content: &T) -> Result<()>
        where T: Editable + Named
    {
        let &Position { line, column } = content.pos();
        let line_count = content.lines().len();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();

        let (screen_width, _) = terminal_size().unwrap();
        let empty_line = (0..screen_width).map(|_| ' ').collect::<String>();
        let y = self.lines_height() as u16;

        try!(self.stdout.style(Style::Invert));

        try!(self.stdout.goto(0, y));
        try!(write!(self.stdout, "{}", empty_line));

        try!(self.stdout.goto(0, y));
        try!(write!(self.stdout, "{}", content.name()));

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width - position_info.len() as u16;
        try!(self.stdout.goto(x, y));
        try!(write!(self.stdout, "{}", position_info));

        try!(self.stdout.style(Style::Reset));
        Ok(())
    }

    fn paint_lines<T: Editable>(&mut self, content: &T) -> Result<()> {
        let line_offset = self.line_offset(&content.pos());
        let window = content.lines()
            .iter()
            .skip(line_offset as usize)
            .take(self.lines_height() as usize)
            .enumerate();

        let mut y = 0;
        for (relative_number, line) in window {
            let absolute_number = relative_number + line_offset as usize;

            try!(self.stdout.goto(0, y));
            try!(write!(self.stdout, "{}", (absolute_number + 1).to_string()));

            let line_start = self.line_number_width(content) as u16 + 1;
            try!(self.stdout.goto(line_start, y));
            try!(write!(self.stdout, "{}", line));
            y += 1;
        }
        Ok(())
    }

    fn cursor_pos<T: Editable>(&self, content: &T) -> (usize, usize) {
        // TODO: column offsetting for long lines
        let &Position { line, column } = content.pos();
        let first_line = self.line_offset(&content.pos());
        let y = line - first_line as usize;
        ((self.line_number_width(content) as usize + 1 + column), y)
    }

    fn line_number_width<T: Editable>(&self, content: &T) -> u16 {
        let ref position = content.pos();
        let max_in_window = self.line_offset(position) + self.lines_height();
        let max_in_text = content.lines().len();
        let max = cmp::min(max_in_window, max_in_text as u16);
        max.to_string().len() as u16
    }

    fn line_offset(&self, position: &Position) -> u16 {
        let line = position.line;
        match line.checked_sub(self.lines_height() as usize / 2) {
            None => 0,
            Some(val) => val as u16,
        }
    }

    fn status_height(&self) -> u16 {
        2
    }

    fn lines_height(&self) -> u16 {
        let (_, screen_height) = terminal_size().unwrap();
        cmp::max(screen_height, self.status_height()) - self.status_height()
    }
}
