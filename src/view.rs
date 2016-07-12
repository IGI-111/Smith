extern crate rustbox;
use rustbox::{Color, RustBox};
use text::{Text, Position};
use std::cmp;

pub struct View<'a> {
    term: &'a RustBox,
}
impl<'a> View<'a> {
    pub fn new(term: &'a RustBox) -> View<'a> {
        View { term: term }
    }

    pub fn render(&self, text: &Text) {
        self.paint_lines(text);
        self.paint_cursor(text);
        self.paint_status(text);
        self.term.present();
    }

    fn paint_cursor(&self, text: &Text) {
        let (x, y) = self.cursor_pos(text);
        self.term.set_cursor(x, y);
    }

    fn paint_status(&self, text: &Text) {
        let &Position { line, column } = text.get_pos();
        let line_count = text.get_lines().len();
        let advance = ((line + 1) as f64 / line_count as f64 * 100.0).floor();
        let filename = text.get_name();

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
                        &filename);

        let position_info = format!("{}% {}/{}: {}", advance, line + 1, line_count, column);
        let x = screen_width - position_info.len();
        self.term.print(x,
                        y,
                        rustbox::RB_REVERSE,
                        Color::Default,
                        Color::Default,
                        &position_info);
    }

    fn paint_lines(&self, text: &Text) {
        let line_offset = self.line_offset(text.get_pos());
        let window = text.get_lines()
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

            let line_start = self.line_number_width(text) + 1;
            self.term.print(line_start,
                            y,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            Color::Default,
                            &line);
            y += 1;
        }
    }

    fn cursor_pos(&self, text: &Text) -> (isize, isize) {
        // TODO: column offsetting for long lines
        let &Position { line, column } = text.get_pos();
        let first_line = self.line_offset(text.get_pos());
        let y = line - first_line;
        ((self.line_number_width(text) + 1 + column) as isize, y as isize)
    }

    fn line_number_width(&self, text: &Text) -> usize {
        let position = text.get_pos();
        let max_in_window = self.line_offset(position) + self.lines_height();
        let max_in_text = text.get_lines().len();
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
