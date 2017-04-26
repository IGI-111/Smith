use state::{Named, Editable, Undoable, Selectable};
use termion::event::{Event, Key, MouseEvent};
use view::View;
use std::cmp;
use clipboard::ClipboardContext;
use super::{State, treat_insert_event};

pub fn treat_selected_event<T>(content: &mut T, view: &mut View, event: Event) -> State
    where T: Selectable + Editable + Named + Undoable
{
    match event {
        Event::Key(Key::Ctrl('c')) => {
            let selection = content.slice_sel();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Ctrl('x')) => {
            let selection = content.slice_sel();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            content.delete_sel();
            view.adjust_view(content.line());

            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Backspace) |
        Event::Key(Key::Delete) => {
            content.delete_sel();
            view.adjust_view(content.line());
            content.reset_sel();
            State::Insert
        }
        Event::Key(Key::Char(_)) => {
            content.delete_sel();
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

pub fn treat_select_event<T>(content: &mut T, view: &mut View, event: Event, origin: usize) -> State
    where T: Editable + Selectable
{
    match event {
        Event::Mouse(MouseEvent::Hold(x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            let sel = (cmp::min(origin, content.pos()), cmp::max(origin, content.pos()));
            content.set_sel(sel);
            State::Select(origin)
        }
        Event::Mouse(MouseEvent::Release(x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            if origin != content.pos() {
                let sel = (cmp::min(origin, content.pos()), cmp::max(origin, content.pos()));
                content.set_sel(sel);
                State::Selected
            } else {
                State::Insert
            }

        }
        _ => State::Select(origin),
    }
}
