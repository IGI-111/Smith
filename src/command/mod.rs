use clipboard::{ClipboardContext, ClipboardProvider};
use data::*;
use std::cmp;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use view::View;

#[derive(Debug, Clone)]
pub enum State {
    Insert,
    Message,
    Prompt(String, String, PromptAction),
    Select(usize),
    Selected,
    Open(String),
    Exit,
}

#[derive(Debug, Clone)]
pub enum PromptAction {
    Save,
    ConfirmExit,
    Open,
    ConfirmOpen(String),
}

const SCROLL_FACTOR: usize = 2;

impl State {
    // Handles a Termion event, consuming the current state and returning the new state
    pub fn handle<T>(self, content: &mut T, view: &mut View, event: Event) -> Self
    where
        T: Editable + Saveable + Undoable + Selectable + Modifiable,
    {
        match self {
            State::Prompt(prompt, message, action) => {
                State::handle_prompt(content, view, event, prompt, message, action)
            }
            State::Select(origin) => State::handle_select(content, view, event, origin),
            State::Insert => State::handle_insert(content, view, event),
            State::Message => State::handle_message(content, view, event),
            State::Selected => State::handle_selected(content, view, event),
            State::Open(_) | State::Exit => panic!("Can't handle exit state"),
        }
    }

    fn handle_message<T>(content: &mut T, view: &mut View, event: Event) -> Self
    where
        T: Editable + Named + Undoable + Modifiable + Saveable,
    {
        view.quiet();
        Self::handle_insert(content, view, event)
    }

    fn handle_insert<T>(content: &mut T, view: &mut View, event: Event) -> Self
    where
        T: Editable + Named + Undoable + Modifiable + Saveable,
    {
        match event {
            Event::Key(Key::Ctrl('q')) | Event::Key(Key::Esc) => {
                if content.was_modified() {
                    let prompt = "Changes not saved do you really want to exit (y/N): ".to_string();
                    let message = "".to_string();
                    view.prompt(&prompt, &message);
                    return State::Prompt(prompt, message, PromptAction::ConfirmExit);
                } else {
                    return State::Exit;
                }
            }
            Event::Key(Key::Ctrl('s')) => {
                if content.name().is_empty() {
                    let prompt = "Save to: ".to_string();
                    view.prompt(&prompt, "");
                    return State::Prompt(prompt, "".to_string(), PromptAction::Save);
                } else {
                    let msg = match content.save() {
                        Err(e) => e.to_string(),
                        Ok(_) => format!("Saved file {}", content.name()),
                    };
                    view.message(&msg);
                    return State::Message;
                }
            }
            Event::Key(Key::Ctrl('o')) => {
                let prompt = "Open file: ".to_string();
                let message = "".to_string();
                view.prompt(&prompt, &message);
                return State::Prompt(prompt, message, PromptAction::Open);
            }
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                let (line, col) = view.translate_coordinates(content, x, y);
                content.move_at(line, col);
                return State::Select(content.pos());
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
        State::Insert
    }

    fn handle_prompt<T>(
        content: &mut T,
        view: &mut View,
        event: Event,
        prompt: String,
        mut message: String,
        action: PromptAction,
    ) -> Self
    where
        T: Editable + Saveable + Modifiable,
    {
        match event {
            Event::Key(Key::Char('\n')) => match action {
                PromptAction::Save => {
                    let msg: String;
                    let old_name = content.name().clone();
                    content.set_name(message.clone());
                    msg = match content.save() {
                        Err(e) => {
                            content.set_name(old_name);
                            e.to_string()
                        }
                        Ok(_) => format!("Saved file {}", message),
                    };
                    view.message(&msg);
                    State::Message
                }
                PromptAction::ConfirmExit => {
                    if message.to_lowercase() == "y" {
                        State::Exit
                    } else {
                        view.message("");
                        State::Message
                    }
                }
                PromptAction::Open => {
                    let filename = message;
                    if content.was_modified() {
                        let prompt =
                            "Changes not saved do you really want to open a new file (y/N): "
                                .to_string();
                        let message = "".to_string();
                        view.prompt(&prompt, &message);
                        State::Prompt(prompt, message, PromptAction::ConfirmOpen(filename))
                    } else {
                        State::Open(filename)
                    }
                }
                PromptAction::ConfirmOpen(filename) => {
                    if message.to_lowercase() == "y" {
                        State::Open(filename)
                    } else {
                        view.message("");
                        State::Message
                    }
                }
            },
            Event::Key(Key::Char(c)) => {
                message.push(c);
                view.prompt(&prompt, &message);
                State::Prompt(prompt, message, action)
            }
            Event::Key(Key::Backspace) | Event::Key(Key::Delete) => {
                message.pop();
                view.prompt(&prompt, &message);
                State::Prompt(prompt, message, action)
            }
            Event::Key(Key::Ctrl('q')) => State::Exit,
            Event::Key(Key::Esc) => {
                view.quiet();
                State::Insert
            }
            _ => State::Prompt(prompt, message, action),
        }
    }

    fn handle_selected<T>(content: &mut T, view: &mut View, event: Event) -> Self
    where
        T: Selectable + Editable + Named + Undoable + Modifiable + Saveable,
    {
        match event {
            Event::Key(Key::Ctrl('c')) => {
                let (beg, end) = content.sel().unwrap();

                let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(selection).unwrap();

                content.reset_sel();
                State::Insert
            }
            Event::Key(Key::Ctrl('x')) => {
                let (beg, end) = content.sel().unwrap();

                let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(selection).unwrap();

                delete_sel(content);
                view.adjust_view(content.line());

                content.reset_sel();
                State::Insert
            }
            Event::Key(Key::Backspace) | Event::Key(Key::Delete) => {
                delete_sel(content);
                view.adjust_view(content.line());
                content.reset_sel();
                State::Insert
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

    fn handle_select<T>(content: &mut T, view: &mut View, event: Event, origin: usize) -> Self
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
                State::Select(origin)
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
                    State::Selected
                } else {
                    State::Insert
                }
            }
            _ => State::Select(origin),
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
