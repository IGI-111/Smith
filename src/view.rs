use state::{Position, Editable, Named};
use std::cmp;
use std::io::{stdout, Stdout, Write};
use termion::{Style, TermWrite, IntoRawMode, RawTerminal, terminal_size};

pub struct View {
    stdout: RawTerminal<Stdout>,
    message: String,
}

impl View {
    pub fn new() -> View {
        let mut stdout = stdout().into_raw_mode().unwrap();
        stdout.clear();
        stdout.show_cursor();
        View {
            stdout: stdout,
            message: String::new(),
        }
    }

    pub fn message(&mut self, message: String) {
        self.message = message;
    }

    pub fn reset_message(&mut self) {
        self.message = String::new();
    }


    pub fn render<T>(&mut self, content: &T)
        where T: Editable + Named
    {
        self.stdout.clear();
        self.paint_lines(content);
        self.paint_status(content);
        self.paint_message();
        self.paint_cursor(content);
        self.stdout.flush();
    }

    pub fn paint_message(&mut self) {
        let y = self.lines_height() + 1;
        self.stdout.goto(0, y as u16);
        write!(self.stdout, "{}", self.message);
        self.stdout.flush();
    }

    fn paint_cursor<T: Editable>(&mut self, content: &T) {
        let (x, y) = self.cursor_pos(content);
        self.stdout.goto(x as u16, y as u16);
    }

    fn paint_status<T>(&mut self, content: &T)
        where T: Editable + Named
    {
        let &Position { line, column } = content.pos();
        let line_count = content.lines().len();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();

        let (screen_width, _) = terminal_size().unwrap();
        let empty_line = (0..screen_width).map(|_| ' ').collect::<String>();
        let y = self.lines_height() as u16;

        self.stdout.style(Style::Invert);

        self.stdout.goto(0, y);
        write!(self.stdout, "{}", empty_line);

        self.stdout.goto(0, y);
        write!(self.stdout, "{}", content.name());

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width - position_info.len() as u16;
        self.stdout.goto(x, y);
        write!(self.stdout, "{}", position_info);

        self.stdout.style(Style::Reset);
    }

    fn paint_lines<T: Editable>(&mut self, content: &T) {
        let line_offset = self.line_offset(&content.pos());
        let window = content.lines()
            .iter()
            .skip(line_offset as usize)
            .take(self.lines_height() as usize)
            .enumerate();

        let mut y = 0;
        for (relative_number, line) in window {
            let absolute_number = relative_number + line_offset as usize;

            self.stdout.goto(0, y);
            write!(self.stdout, "{}", (absolute_number + 1).to_string());

            let line_start = self.line_number_width(content) as u16 + 1;
            self.stdout.goto(line_start, y);
            write!(self.stdout, "{}", line);
            y += 1;
        }
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
