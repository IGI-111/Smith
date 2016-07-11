extern crate rustbox;
use rustbox::Color;

pub struct View<'a> {
    buf: &'a str,
    pos: (u32, u32)
}
impl<'a> View<'a> {
    pub fn new() -> View<'a> {
        View {buf:"hello world", pos:(0,0)}
    }
    pub fn paint(self: &View<'a>, term: &rustbox::RustBox) {
        term.print(0, 0,
                   rustbox::RB_NORMAL,
                   Color::White,
                   Color::Black,
                   self.buf);
        term.present();
    }
}

