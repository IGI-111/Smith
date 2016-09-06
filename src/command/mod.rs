use state::{Named, Editable, Undoable, Saveable, Selectable, Movement};
use termion::event::{Event, Key, MouseEvent, MouseButton};
use view::View;
use std::cmp;
use clipboard::ClipboardContext;

enum State {
    Insert,
    Message,
    Prompt(String, String),
    Exit,
    Select(usize),
    Selected,
}

pub struct Command {
    state: State,
}

impl Command {
    pub fn new() -> Command {
        Command { state: State::Insert }
    }

    pub fn treat_event<T>(&mut self, content: &mut T, view: &mut View, event: Event) -> bool
        where T: Editable + Saveable + Undoable + Selectable
        {
            match self.state {
                State::Insert => treat_insert_event(content, view, event, &mut self.state),
                State::Prompt(_, _) => treat_prompt_event(content, view, event, &mut self.state),
                State::Message => treat_message_event(content, view, event, &mut self.state),
                State::Select(_) => treat_select_event(content, view, event, &mut self.state),
                State::Selected => treat_selected_event(content, view, event, &mut self.state),
                State::Exit => panic!("continued after an Exit state"),
            };
            if let State::Exit = self.state {
                return true;
            }
            false
        }
}

fn treat_message_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Named + Undoable
{
    view.quiet();
    treat_insert_event(content, view, event, state)
}

fn treat_selected_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Selectable + Editable + Named + Undoable
{
    let res = match event {
        // TODO remove, copy etc
        _ => {
            treat_insert_event(content, view, event, state);
            *state = State::Insert;
        }
    };
    content.reset_sel();
    res
}

fn treat_insert_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Named + Undoable
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
        Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            *state = State::Select(content.pos());
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
}

fn treat_prompt_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Saveable
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
            *state = State::Message;
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


fn treat_select_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Selectable
{
    if let Event::Mouse(MouseEvent::Release(x, y)) = event {
        let (line, col) = view.translate_coordinates(content, x, y);
        content.move_at(line, col);
        if let State::Select(origin) = *state {
            let sel = (cmp::min(origin, content.pos()), cmp::max(origin, content.pos()));
            content.set_sel(sel);
        } else {
            panic!("Treating select event when event is not a Select");
        }

        *state = State::Selected;
    }
}
