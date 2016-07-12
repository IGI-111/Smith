use text::{Text, Movement};
use rustbox::{Event, Key};

pub fn treat_event(text: &mut Text, event: &Event) -> bool {
    match event {
        &Event::KeyEvent(key) => {
            match key {
                Key::Ctrl('q') => {
                    return true;
                }
                Key::Up => {
                    text.step(Movement::Up);
                }
                Key::Down => {
                    text.step(Movement::Down);
                }
                Key::Left => {
                    text.step(Movement::Left);
                }
                Key::Right => {
                    text.step(Movement::Right);
                }
                Key::Home => {
                    text.step(Movement::LineStart);
                }
                Key::End => {
                    text.step(Movement::LineEnd);
                }
                Key::Backspace => {
                    text.delete();
                }
                Key::Enter => {
                    text.new_line();
                }
                Key::Char(c) => {
                    text.insert(c);
                }
                _ => {}
            }
        }
        _ => {}
    }
    false
}
