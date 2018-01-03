mod select;

use state::{Named, Editable, Undoable, Saveable, Selectable, Movement};
use termion::event::{Event, Key, MouseEvent, MouseButton};
use view::View;
use clipboard::ClipboardContext;
use self::select::{treat_select_event, treat_selected_event};

#[derive(Debug, Clone)]
pub enum State {
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

const SCROLL_FACTOR: usize = 2;

impl Command {
    pub fn new() -> Command {
        Command { state: State::Insert }
    }

    pub fn treat_event<T>(&mut self, content: &mut T, view: &mut View, event: Event) -> bool
    where
        T: Editable + Saveable + Undoable + Selectable,
    {
        self.state = match self.state.clone() {
            State::Prompt(prompt, message) => {
                treat_prompt_event(content, view, event, prompt, message)
            }
            State::Select(origin) => treat_select_event(content, view, event, origin),
            State::Insert => treat_insert_event(content, view, event),
            State::Message => treat_message_event(content, view, event),
            State::Selected => treat_selected_event(content, view, event),
            State::Exit => panic!("continued after an Exit state"),
        };
        if let State::Exit = self.state {
            true
        } else {
            false
        }
    }
}

pub fn treat_message_event<T>(content: &mut T, view: &mut View, event: Event) -> State
where
    T: Editable + Named + Undoable,
{
    view.quiet();
    treat_insert_event(content, view, event)
}

pub fn treat_insert_event<T>(content: &mut T, view: &mut View, event: Event) -> State
where
    T: Editable + Named + Undoable,
{
    match event {
        Event::Key(Key::Ctrl('q')) |
        Event::Key(Key::Esc) => State::Exit,
        Event::Key(Key::Ctrl('s')) => {
            let prompt = "Save to: ".to_string();
            let message = content.name().to_string();
            view.prompt(&prompt, &message);
            State::Prompt(prompt, message)
        }
        Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            State::Select(content.pos())
        }
        Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
            view.scroll_view(SCROLL_FACTOR as isize, content);
            State::Insert
        }
        Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
            view.scroll_view(-(SCROLL_FACTOR as isize), content);
            State::Insert
        }
        Event::Key(Key::Ctrl('z')) => {
            content.undo();
            State::Insert
        }
        Event::Key(Key::Ctrl('y')) => {
            content.redo();
            State::Insert
        }
        Event::Key(Key::Ctrl('v')) => {
            let ctx = ClipboardContext::new().unwrap();
            for c in ctx.get_contents().unwrap().chars() {
                content.insert(c);
            }
            State::Insert
        }
        Event::Key(Key::Up) => {
            content.step(Movement::Up);
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::Down) => {
            content.step(Movement::Down);
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::Left) => {
            content.step(Movement::Left);
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::Right) => {
            content.step(Movement::Right);
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::PageUp) => {
            content.step(Movement::PageUp(view.lines_height() as usize));
            view.center_view(content.line());
            State::Insert
        }
        Event::Key(Key::PageDown) => {
            content.step(Movement::PageDown(view.lines_height() as usize));
            view.center_view(content.line());
            State::Insert
        }
        Event::Key(Key::Home) => {
            content.step(Movement::LineStart);
            State::Insert
        }
        Event::Key(Key::End) => {
            content.step(Movement::LineEnd);
            State::Insert
        }
        Event::Key(Key::Backspace) |
        Event::Key(Key::Ctrl('h')) => {
            content.delete();
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::Delete) => {
            content.delete_forward();
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Key(Key::Char(c)) => {
            content.insert(c);
            view.adjust_view(content.line());
            State::Insert
        }
        Event::Unsupported(u) => {
            view.message(&format!("Unsupported escape sequence {:?}", u));
            State::Insert
        }
        _ => State::Insert,
    }
}

fn treat_prompt_event<T>(
    content: &mut T,
    view: &mut View,
    event: Event,
    prompt: String,
    mut message: String,
) -> State
where
    T: Editable + Saveable,
{
    match event {
        Event::Key(Key::Char('\n')) => {
            let msg: String;
            let old_name = content.name().clone();
            content.set_name(message);
            msg = match content.save() {
                Err(e) => {
                    content.set_name(old_name);
                    e.to_string()
                }
                Ok(_) => format!("Saved file {}", content.name()),
            };
            view.message(&msg);
            State::Message
        }
        Event::Key(Key::Char(c)) => {
            message.push(c);
            view.prompt(&prompt, &message);
            State::Prompt(prompt, message)
        }
        Event::Key(Key::Backspace) |
        Event::Key(Key::Delete) => {
            message.pop();
            view.prompt(&prompt, &message);
            State::Prompt(prompt, message)
        }
        Event::Key(Key::Ctrl('q')) => State::Exit,
        Event::Key(Key::Esc) => {
            view.quiet();
            State::Insert
        }
        _ => State::Prompt(prompt, message),
    }
}
