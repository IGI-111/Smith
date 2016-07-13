use state::{Text, Editable, Named, Saveable, Movement};
use rustbox::{Event, Key};
use view::View;

pub fn treat_event<T>(content: &mut T, view: &View, event: &Event) -> bool
where T: Editable + Saveable {
    match event {
        &Event::KeyEvent(key) => {
            match key {
                Key::Ctrl('q') => { return true; }
                Key::Ctrl('s') => {
                    match content.save() {
                        Err(e) => view.render_message(e.to_string()),
                        Ok(_) => view.render_message(format!("Saved file {}", content.name())),
                    }
                }
                Key::Up => { content.step(Movement::Up); }
                Key::Down => { content.step(Movement::Down); }
                Key::Left => { content.step(Movement::Left); }
                Key::Right => { content.step(Movement::Right); }
                Key::Home => { content.step(Movement::LineStart); }
                Key::End => { content.step(Movement::LineEnd); }
                Key::Backspace => { content.delete(); }
                Key::Enter => { content.insert('\n'); }
                Key::Char(c) => { content.insert(c); }
                _ => {}
            }
        }
        _ => {}
    }
    false
}
