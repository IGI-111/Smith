extern crate rustbox;
use rustbox::{Color, RustBox};
use state::{Position, Editable, Named};
use std::cmp;

pub struct View<'a> {
    term: &'a RustBox,
}
impl<'a> View<'a> {
    pub fn new(term: &'a RustBox) -> View<'a> {
        View { term: term }
    }

    pub fn render_message(&self, message: String) {
        let y = self.lines_height() + 1;
        self.term.print(0,
                        y,
                        rustbox::RB_NORMAL,
                        Color::Default,
                        Color::Default,
                        &message);
        self.term.present();
    }

    pub fn render<T>(&self, content: &T)
        where T: Editable + Named
    {
        self.paint_lines(content);
        self.paint_cursor(content);
        self.paint_status(content);
        self.term.present();
    }

    fn paint_cursor<T: Editable>(&self, content: &T) {
        let (x, y) = self.cursor_pos(content);
        self.term.set_cursor(x, y);
    }

    fn paint_status<T>(&self, content: &T)
        where T: Editable + Named
    {
        let &Position { line, column } = content.pos();
        let line_count = content.lines().len();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();

        let screen_width = self.term.width();
        let empty_line = (0..screen_width).map(|_| ' ').collect::<String>();
        let y = self.lines_height();
        self.term.print(0,
                        y,
                        rustbox::RB_REVERSE,
                        Color::Default,
                        Color::Default,
                        &empty_line);

        self.term.print(0,
                        y,
                        rustbox::RB_REVERSE,
                        Color::Default,
                        Color::Default,
                        &content.name());

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width - position_info.len();
        self.term.print(x,
                        y,
                        rustbox::RB_REVERSE,
                        Color::Default,
                        Color::Default,
                        &position_info);
    }

    fn paint_lines<T: Editable>(&self, content: &T) {
        let line_offset = self.line_offset(&content.pos());
        let window = content.lines()
            .iter()
            .skip(line_offset)
            .take(self.lines_height())
            .enumerate();

        let mut y = 0;
        for (relative_number, line) in window {
            let absolute_number = relative_number + line_offset;
            self.term.print(0,
                            y,
                            rustbox::RB_NORMAL,
                            Color::White,
                            Color::Default,
                            &(absolute_number + 1).to_string());

            let line_start = self.line_number_width(content) + 1;
            self.term.print(line_start,
                            y,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            Color::Default,
                            &line);
            y += 1;
        }
    }

    fn cursor_pos<T: Editable>(&self, content: &T) -> (isize, isize) {
        // TODO: column offsetting for long lines
        let &Position { line, column } = content.pos();
        let first_line = self.line_offset(&content.pos());
        let y = line - first_line;
        ((self.line_number_width(content) + 1 + column) as isize, y as isize)
    }

    fn line_number_width<T: Editable>(&self, content: &T) -> usize {
        let ref position = content.pos();
        let max_in_window = self.line_offset(position) + self.lines_height();
        let max_in_text = content.lines().len();
        let max = cmp::min(max_in_window, max_in_text);
        max.to_string().len()
    }

    fn line_offset(&self, position: &Position) -> usize {
        let line = position.line;
        match line.checked_sub(self.lines_height() / 2) {
            None => 0,
            Some(val) => val,
        }
    }

    fn status_height(&self) -> usize {
        2
    }

    fn lines_height(&self) -> usize {
        self.term.height() - self.status_height()
    }
}
