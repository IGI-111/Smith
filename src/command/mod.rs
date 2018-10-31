use clipboard::{ClipboardContext, ClipboardProvider};
use data::{Editable, Movement, Named, Saveable, Selectable, Undoable};
use std::cmp;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use view::View;

#[derive(Debug, Clone)]
pub enum State {
    Insert,
    Message,
    Prompt(String, String),
    Select(usize),
    Selected,
}

const SCROLL_FACTOR: usize = 2;

impl State {
    // Handles a Termion event, consuming the current state and returning the new state
    pub fn handle<T>(self, content: &mut T, view: &mut View, event: Event) -> Option<Self>
    where
        T: Editable + Saveable + Undoable + Selectable,
    {
        match self {
            State::Prompt(prompt, message) => {
                State::handle_prompt(content, view, event, prompt, message)
            }
            State::Select(origin) => State::handle_select(content, view, event, origin),
            State::Insert => State::handle_insert(content, view, event),
            State::Message => State::handle_message(content, view, event),
            State::Selected => State::handle_selected(content, view, event),
        }
    }

    fn handle_message<T>(content: &mut T, view: &mut View, event: Event) -> Option<Self>
    where
        T: Editable + Named + Undoable,
    {
        view.quiet();
        Self::handle_insert(content, view, event)
    }

    fn handle_insert<T>(content: &mut T, view: &mut View, event: Event) -> Option<Self>
    where
        T: Editable + Named + Undoable,
    {
        match event {
            Event::Key(Key::Ctrl('q')) | Event::Key(Key::Esc) => return None,
            Event::Key(Key::Ctrl('s')) => {
                let prompt = "Save to: ".to_string();
                let message = content.name().to_string();
                view.prompt(&prompt, &message);
                return Some(State::Prompt(prompt, message));
            }
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                let (line, col) = view.translate_coordinates(content, x, y);
                content.move_at(line, col);
                return Some(State::Select(content.pos()));
            }
            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                view.scroll_view(SCROLL_FACTOR as isize, content);
            }
            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                view.scroll_view(-(SCROLL_FACTOR as isize), content);
            }
            Event::Key(Key::Ctrl('z')) => {
                content.undo();
            }
            Event::Key(Key::Ctrl('y')) => {
                content.redo();
            }
            Event::Key(Key::Ctrl('v')) => {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                for c in ctx
                    .get_contents()
                    .unwrap_or_else(|_| "".to_string())
                    .chars()
                {
                    content.insert(c);
                }
            }
            Event::Key(Key::Up) => {
                content.step(Movement::Up);
                view.adjust_view(content.line());
            }
            Event::Key(Key::Down) => {
                content.step(Movement::Down);
                view.adjust_view(content.line());
            }
            Event::Key(Key::Left) => {
                content.step(Movement::Left);
                view.adjust_view(content.line());
            }
            Event::Key(Key::Right) => {
                content.step(Movement::Right);
                view.adjust_view(content.line());
            }
            Event::Key(Key::PageUp) => {
                content.step(Movement::PageUp(view.lines_height() as usize));
                view.center_view(content.line());
            }
            Event::Key(Key::PageDown) => {
                content.step(Movement::PageDown(view.lines_height() as usize));
                view.center_view(content.line());
            }
            Event::Key(Key::Home) => {
                content.step(Movement::LineStart);
            }
            Event::Key(Key::End) => {
                content.step(Movement::LineEnd);
            }
            Event::Key(Key::Backspace) | Event::Key(Key::Ctrl('h')) => {
                content.delete();
                view.adjust_view(content.line());
            }
            Event::Key(Key::Delete) => {
                content.delete_forward();
                view.adjust_view(content.line());
            }
            Event::Key(Key::Char(c)) => {
                content.insert(c);
                view.adjust_view(content.line());
            }
            Event::Unsupported(u) => {
                view.message(&format!("Unsupported escape sequence {:?}", u));
            }
            _ => {}
        }
        Some(State::Insert)
    }

    fn handle_prompt<T>(
        content: &mut T,
        view: &mut View,
        event: Event,
        prompt: String,
        mut message: String,
    ) -> Option<Self>
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
                Some(State::Message)
            }
            Event::Key(Key::Char(c)) => {
                message.push(c);
                view.prompt(&prompt, &message);
                Some(State::Prompt(prompt, message))
            }
            Event::Key(Key::Backspace) | Event::Key(Key::Delete) => {
                message.pop();
                view.prompt(&prompt, &message);
                Some(State::Prompt(prompt, message))
            }
            Event::Key(Key::Ctrl('q')) => None,
            Event::Key(Key::Esc) => {
                view.quiet();
                Some(State::Insert)
            }
            _ => Some(State::Prompt(prompt, message)),
        }
    }

    fn handle_selected<T>(content: &mut T, view: &mut View, event: Event) -> Option<Self>
    where
        T: Selectable + Editable + Named + Undoable,
    {
        match event {
            Event::Key(Key::Ctrl('c')) => {
                let (beg, end) = content.sel().unwrap();

                let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(selection).unwrap();

                content.reset_sel();
                Some(State::Insert)
            }
            Event::Key(Key::Ctrl('x')) => {
                let (beg, end) = content.sel().unwrap();

                let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(selection).unwrap();

                delete_sel(content);
                view.adjust_view(content.line());

                content.reset_sel();
                Some(State::Insert)
            }
            Event::Key(Key::Backspace) | Event::Key(Key::Delete) => {
                delete_sel(content);
                view.adjust_view(content.line());
                content.reset_sel();
                Some(State::Insert)
            }
            Event::Key(Key::Char(_)) => {
                delete_sel(content);
                view.adjust_view(content.line());
                content.reset_sel();
                Self::handle_insert(content, view, event)
            }
            _ => {
                content.reset_sel();
                Self::handle_insert(content, view, event)
            }
        }
    }

    fn handle_select<T>(
        content: &mut T,
        view: &mut View,
        event: Event,
        origin: usize,
    ) -> Option<Self>
    where
        T: Editable + Selectable,
    {
        match event {
            Event::Mouse(MouseEvent::Hold(x, y)) => {
                let (line, col) = view.translate_coordinates(content, x, y);
                content.move_at(line, col);
                let sel = (
                    cmp::min(origin, content.pos()),
                    cmp::max(origin, content.pos()),
                );
                content.set_sel(sel);
                Some(State::Select(origin))
            }
            Event::Mouse(MouseEvent::Release(x, y)) => {
                let (line, col) = view.translate_coordinates(content, x, y);
                content.move_at(line, col);
                if origin != content.pos() {
                    let sel = (
                        cmp::min(origin, content.pos()),
                        cmp::max(origin, content.pos()),
                    );
                    content.set_sel(sel);
                    Some(State::Selected)
                } else {
                    Some(State::Insert)
                }
            }
            _ => Some(State::Select(origin)),
        }
    }
}

fn delete_sel<T>(content: &mut T)
where
    T: Selectable + Editable,
{
    let (beg, end) = content.sel().unwrap();
    assert!(beg < end);
    let end = cmp::min(end + 1, content.len() - 1);
    content.move_to(end);
    for _ in beg..end {
        content.delete();
    }
}
