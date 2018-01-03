use state::{Named, Editable, Undoable, Selectable};
use termion::event::{Event, Key, MouseEvent};
use view::View;
use std::cmp;
use clipboard::ClipboardContext;
use super::{State, treat_insert_event};

pub fn treat_selected_event<T>(content: &mut T, view: &mut View, event: Event) -> State
where
    T: Selectable + Editable + Named + Undoable,
{
    match event {
        Event::Key(Key::Ctrl('c')) => {
            let (beg, end) = content.sel().unwrap();

            let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Ctrl('x')) => {
            let (beg, end) = content.sel().unwrap();

            let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            delete_sel(content);
            view.adjust_view(content.line());

            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Backspace) |
        Event::Key(Key::Delete) => {
            delete_sel(content);
            view.adjust_view(content.line());
            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Char(_)) => {
            delete_sel(content);
            view.adjust_view(content.line());
            content.reset_sel();
            treat_insert_event(content, view, event)
        }
        _ => {
            content.reset_sel();
            treat_insert_event(content, view, event)
        }
    }
}

fn delete_sel<T>(content: &mut T)
where
    T: Selectable + Editable,
{
    let (beg, end) = content.sel().unwrap();
    let end = cmp::min(end + 1, content.len() - 1);
    content.move_to(end);
    for _ in beg..end {
        content.delete();
    }
}


pub fn treat_select_event<T>(content: &mut T, view: &mut View, event: Event, origin: usize) -> State
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
