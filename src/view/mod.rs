use state::{Selectable, Editable, Named};
use std::cmp;
use std::io::{stdout, Stdout, BufWriter, Write, Result};
use std::iter;
use termion::terminal_size;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, style, cursor, color};

pub struct View {
    stdout: BufWriter<MouseTerminal<RawTerminal<Stdout>>>,
    message: Option<String>,
    is_prompt: bool,
    line_offset: u16,
}

const TAB_LENGTH: usize = 4;

    impl Drop for View {
    fn drop(&mut self) {
        write!(self.stdout, "\x1B[r\x1B[?1049l").unwrap();
    }
}

impl View {
    pub fn new() -> Result<View> {
        let mut stdout = BufWriter::new(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
        try!(write!(stdout, "\x1B[?1049h"));
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
        let line = cmp::min((y as isize + self.line_offset as isize - 1) as usize,
                            content.line_count() - 1);
        let visual_col = (cmp::max(0,
                                   x as isize -
                                   self.line_number_width(content.line_count()) as isize -
                                   2)) as usize;
        // find out if we clicked through a tab
        let col = content.iter_line(line)
            .scan(0, |state, x| {
                *state += if x == '\t' { TAB_LENGTH } else { 1 };
                Some(*state)
            })
            .take_while(|&x| x <= visual_col)
            .count();
        (line, col)
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
            (self.message.clone().unwrap().chars().count() as u16, self.lines_height() + 1)
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
        let line_offset = self.line_offset as usize;
        let lines_height = self.lines_height() as usize;
        let lines_width = self.lines_width(content.line_count()) as usize;
        let line_count = content.line_count();

        let line_start = self.line_number_width(line_count) + 1;

        let mut chars = content.line_index_to_char_index(line_offset);
        for y in 0..cmp::min(lines_height, line_count - line_offset) {
            let line = y + line_offset;
            try!(write!(self.stdout,
                        "{}{}{}{}{}",
                        color::Fg(color::White),
                        cursor::Goto(1, 1 + y as u16),
                        1 + line,
                        style::Reset,
                        cursor::Goto(1 + line_start, 1 + y as u16)));

            let mut line_len: usize = 0;
            for c in content.iter_line(line).take_while(|&x| x != '\n') {
                if line_len > lines_width - 1 {
                    // do nothing but count
                    chars += 1;
                    line_len += 1;
                } else {
                    if content.in_sel(chars) {
                        try!(write!(self.stdout, "{}", style::Invert));
                    }
                    if c == '\t' {
                        try!(write!(self.stdout,
                                    "{}",
                                    iter::repeat(" ").take(TAB_LENGTH).collect::<String>()));
                        line_len += TAB_LENGTH;
                    } else {
                        try!(write!(self.stdout, "{}", c));
                        line_len += 1;
                    }
                    try!(write!(self.stdout, "{}", style::Reset));
                    chars += 1;
                }
            }
            chars += 1;
            if line_len <= 1 && content.in_sel(chars) {
                try!(write!(self.stdout, "{} {}", style::Invert, style::Reset));
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
        let visual_col = content.col();
        let column = content.iter_line(line)
            .map(|x| if x == '\t' { TAB_LENGTH } else { 1 })
            .take(visual_col)
            .fold(0, |acc, x| acc + x);
        ((self.line_number_width(content.line_count()) as usize + 1 + column), y)
    }

    fn line_number_width(&self, line_count: usize) -> u16 {
        line_count.to_string().len() as u16
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
