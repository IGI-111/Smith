use state::{Editable, Undoable, Saveable, Movement};
use termion::{Key, TermWrite};
use view::View;
use clipboard::ClipboardContext;

pub fn treat_event<T>(content: &mut T, view: &mut View, key: Key) -> bool
    where T: Editable + Saveable + Undoable
{
    match key {
        Key::Ctrl('s') => {
            match content.save() {
                Err(e) => view.message(e.to_string()),
                Ok(_) => view.message(format!("Saved file {}", content.name())),
            }
        }
        key => {
            view.reset_message();
            match key {
                Key::Ctrl('q') => {
                    return true;
                }
                Key::Ctrl('z') => {
                    content.undo();
                }
                Key::Ctrl('y') => {
                    content.redo();
                }
                Key::Ctrl('v') => {
                    let ctx = ClipboardContext::new().unwrap();
                    for c in ctx.get_contents().unwrap().chars() {
                        content.insert(c);
                    }
                }
                Key::Up => {
                    content.step(Movement::Up);
                }
                Key::Down => {
                    content.step(Movement::Down);
                }
                Key::Left => {
                    content.step(Movement::Left);
                }
                Key::Right => {
                    content.step(Movement::Right);
                }
                // Key::Home => {
                //     content.step(Movement::LineStart);
                // }
                // Key::End => {
                //     content.step(Movement::LineEnd);
                // }
                Key::Backspace => {
                    content.delete();
                }
                Key::Char(c) => {
                    content.insert(c);
                }
                _ => {}
            }
        }
    }
    false
}
