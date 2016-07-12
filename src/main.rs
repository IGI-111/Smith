extern crate rustbox;
mod view;
mod text;

use std::default::Default;

use rustbox::RustBox;
use rustbox::Key;
use text::{Text,Movement};
use view::View;

fn main() {
    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };
    let mut text = Text::new();
    let view = View::new(&rustbox);

    rustbox.clear();
    view.render(&text);

    loop {
        match rustbox.poll_event(false) {
            Ok(event) => {
                match event{
                    rustbox::Event::KeyEvent(key) => match key {
                        Key::Ctrl('q') => { break; }
                        Key::Up => { text.step(Movement::Up); }
                        Key::Down => { text.step(Movement::Down); }
                        Key::Left => { text.step(Movement::Left); }
                        Key::Right => { text.step(Movement::Right); }
                        Key::Home => { text.step(Movement::LineStart); }
                        Key::End => { text.step(Movement::LineEnd); }
                        Key::Backspace => { text.delete(); }
                        Key::Enter => { text.new_line(); }
                        Key::Char(c) => { text.insert(c); }
                        _ => { }
                    },
                    _ => {}
                }
                rustbox.clear();
                view.render(&text);
            },
            Err(e) => panic!("{}", e),
        }
    }
}
