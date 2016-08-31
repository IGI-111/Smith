use state::{Editable, Undoable, Saveable, Movement};
use termion::event::Key;
use view::View;
use clipboard::ClipboardContext;

enum State {
    Insert,
    Message(String),
    Prompt(String),
    Exit,
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
        let next_state = match self.state {
            State::Insert => self.treat_insert_event(content, view, key),
            State::Prompt(ref current_message) => {
                self.treat_prompt_event(content, view, key, current_message)
            }
            State::Message(_) => self.treat_message_event(content, view, key),
            State::Exit => panic!("continued after an Exit state"),
        };
        if let State::Exit = next_state { return true; }
        self.state = next_state;
        false
    }

    fn treat_message_event<T>(&self, content: &mut T, view: &mut View, key: Key) -> State
        where T: Editable + Saveable + Undoable
    {
        view.quiet();
        self.treat_insert_event(content, view, key)
    }

    fn treat_insert_event<T>(&self, content: &mut T, view: &mut View, key: Key) -> State
        where T: Editable + Saveable + Undoable
    {
        match key {
            Key::Ctrl('q') => State::Exit,
            Key::Ctrl('s') => {
                let msg = match content.save() {
                    Err(e) => e.to_string(),
                    Ok(_) => format!("Saved file {}", content.name()),
                };
                view.message(msg.clone());
                State::Message(msg)
            }
            key => {
                match key {
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
                    Key::PageUp => content.step(Movement::PageUp(view.lines_height() as usize)),
                    Key::PageDown => content.step(Movement::PageDown(view.lines_height() as usize)),
                    Key::Home => content.step(Movement::LineStart),
                    Key::End => content.step(Movement::LineEnd),
                    Key::Backspace => {
                        content.delete();
                    }
                    Key::Char(c) => content.insert(c),
                    _ => {}
                }
                State::Insert
            }
        }
    }
    fn treat_prompt_event<T>(&self,
                             content: &mut T,
                             view: &mut View,
                             key: Key,
                             current_message: &String)
                             -> State {
        match key {
            _ => {}
        };
        State::Exit
    }
}
