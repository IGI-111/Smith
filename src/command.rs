use state::{Editable, Undoable, Saveable, Movement};
use termion::event::{Event, Key, MouseEvent, MouseButton};
use view::View;
use clipboard::ClipboardContext;
use std::cmp;

enum State {
    Insert,
    Message(String),
    Prompt(String, String),
    Exit,
}

pub struct Command {
    state: State,
}

impl Command {
    pub fn new() -> Command {
        Command { state: State::Insert }
    }

    pub fn treat_event<T>(&mut self, content: &mut T, view: &mut View, event: Event) -> bool
        where T: Editable + Saveable + Undoable
    {
        match self.state {
            State::Insert => treat_insert_event(content, view, event, &mut self.state),
            State::Prompt(_, _) => treat_prompt_event(content, view, event, &mut self.state),
            State::Message(_) => treat_message_event(content, view, event, &mut self.state),
            State::Exit => panic!("continued after an Exit state"),
        };
        if let State::Exit = self.state {
            return true;
        }
        false
    }
}

fn treat_message_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    view.quiet();
    treat_insert_event(content, view, event, state)
}

fn treat_insert_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    match event {
        Event::Key(Key::Ctrl('q')) => {
            *state = State::Exit;
        }
        Event::Key(Key::Ctrl('s')) => {
            let prompt = "Save to: ".to_string();
            let message = content.name().clone();
            view.prompt(&prompt, &message);
            *state = State::Prompt(prompt, message);
        }
        event => {
            match event {
                Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                    // FIXME: this should be handled somewhere else (in the view?)
                    let line = y as isize + view.line_offset(content.line()) as isize - 1;
                    let col = cmp::max(0, x as isize - view.line_number_width(content.line(), content.line_count()) as isize - 2);
                    content.move_at(line as usize, col as usize);
                }
                Event::Key(Key::Ctrl('z')) => content.undo(),
                Event::Key(Key::Ctrl('y')) => content.redo(),
                Event::Key(Key::Ctrl('v')) => {
                    let ctx = ClipboardContext::new().unwrap();
                    for c in ctx.get_contents().unwrap().chars() {
                        content.insert(c);
                    }
                }
                Event::Key(Key::Up) => content.step(Movement::Up),
                Event::Key(Key::Down) => content.step(Movement::Down),
                Event::Key(Key::Left) => content.step(Movement::Left),
                Event::Key(Key::Right) => content.step(Movement::Right),
                Event::Key(Key::PageUp) => {
                    content.step(Movement::PageUp(view.lines_height() as usize))
                }
                Event::Key(Key::PageDown) => {
                    content.step(Movement::PageDown(view.lines_height() as usize))
                }
                Event::Key(Key::Home) => content.step(Movement::LineStart),
                Event::Key(Key::End) => content.step(Movement::LineEnd),
                Event::Key(Key::Backspace) => {
                    content.delete();
                }
                Event::Key(Key::Char(c)) => content.insert(c),
                _ => {}
            }
            *state = State::Insert;
        }
    }
}
fn treat_prompt_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Saveable + Undoable
{
    match event {
        Event::Key(Key::Char('\n')) => {
            let msg: String;
            if let State::Prompt(_, ref mut message) = *state {
                let old_name = content.name().clone();
                content.set_name(message.clone());
                msg = match content.save() {
                    Err(e) => {
                        content.set_name(old_name);
                        e.to_string()
                    }
                    Ok(_) => format!("Saved file {}", content.name()),
                };
                view.message(&msg);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
            *state = State::Message(msg);
        }
        Event::Key(Key::Char(c)) => {
            if let State::Prompt(ref prompt, ref mut message) = *state {
                message.push(c);
                view.prompt(prompt, message);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
        }
        Event::Key(Key::Backspace) => {
            if let State::Prompt(ref prompt, ref mut message) = *state {
                message.pop();
                view.prompt(prompt, message);
            } else {
                panic!("Treating prompt event when event is not a Prompt");
            }
        }
        Event::Key(Key::Ctrl('q')) => {
            *state = State::Exit;
        }
        _ => {}
    }
}
