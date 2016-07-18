use state::{Editable, Undoable, Saveable, Movement};
use termion::Key;
use view::View;
use clipboard::ClipboardContext;

enum State {
    Insert,
    Prompt,
}

pub struct Command {
    state: State,
}

impl Command {
    pub fn new() -> Command {
        Command { state: State::Insert }
    }

    pub fn treat_event<T>(&mut self, content: &mut T, view: &mut View, key: Key) -> bool
        where T: Editable + Saveable + Undoable
    {
        match self.state {
            State::Insert => self.treat_insert_event(content, view, key),
            State::Prompt => self.treat_prompt_event(content, view, key),
        }
    }
    pub fn treat_insert_event<T>(&mut self, content: &mut T, view: &mut View, key: Key) -> bool
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
                view.quiet();
                match key {
                    Key::Ctrl('q') => return true,
                    Key::Ctrl('z') => content.undo(),
                    Key::Ctrl('y') => content.redo(),
                    Key::Ctrl('v') => {
                        let ctx = ClipboardContext::new().unwrap();
                        for c in ctx.get_contents().unwrap().chars() {
                            content.insert(c);
                        }
                    }
                    Key::Up => content.step(Movement::Up),
                    Key::Down => content.step(Movement::Down),
                    Key::Left => content.step(Movement::Left),
                    Key::Right => content.step(Movement::Right),
                    Key::Home => content.step(Movement::LineStart),
                    Key::End => content.step(Movement::LineEnd),
                    Key::Backspace => { content.delete(); },
                    Key::Char(c) => content.insert(c),
                    _ => {}
                };
            }
        }
        false
    }
    pub fn treat_prompt_event<T>(&mut self, content: &mut T, view: &mut View, key: Key) -> bool { true }
}
