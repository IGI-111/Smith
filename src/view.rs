extern crate rustbox;
use rustbox::{Color, RustBox};
use text::Text;

pub struct View<'a> {
    term: &'a RustBox,
}
impl<'a> View<'a> {
    pub fn new(term: &'a RustBox) -> View<'a> {
        View {term: term}
    }

    pub fn paint(&self, text: &Text) {
        self.paint_text(text);
        self.paint_cursor(text);
        self.paint_status(text);
        self.term.present();
    }

    fn paint_cursor(&self, text: &Text) {
        let (x, y) = self.cursor_pos(text);
        self.term.set_cursor(x, y);
    }

    fn paint_status(&self, text: &Text) {
        let (line, col) = text.get_pos();
        let (x, y) = self.cursor_pos(text);
        self.term.print(0, self.term.height()-1, rustbox::RB_REVERSE, Color::Default, Color::Default,
                        &format!("({}, {}) {}, {}", x, y, line, col));
    }

    fn paint_text(&self, text: &Text) {
        let mut i = 0;
        for line in text.get_lines().iter() {
            self.term.print(0, i, rustbox::RB_NORMAL, Color::Default, Color::Default,
                            &line);
            i += 1;
        }
    }

    fn cursor_pos(&self, text: &Text) -> (isize, isize) {
        //FIXME: column used is wrong
        let (line, column) = text.get_pos();
        let first_line = self.first_line(text);
        let y = line + first_line;
        (column as isize, y as isize)
    }

    fn first_line(&self, text: &Text) -> usize {
        let line = text.get_pos().0;
        let screen_height = self.term.height();

        match line.checked_sub(screen_height / 2) {
            None => 0,
            Some(val) => val,
        }
    }

    fn last_line(&self, text: &Text) -> usize {
        self.first_line(text) + self.term.height()
    }
}

