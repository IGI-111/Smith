use state::{Named, Editable, Undoable, Selectable};
use termion::event::{Event, Key, MouseEvent};
use view::View;
use std::cmp;
use clipboard::ClipboardContext;
use super::{State, treat_insert_event};

pub fn treat_selected_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Selectable + Editable + Named + Undoable
{
    match event {
        Event::Key(Key::Ctrl('c')) => {
            let (beg, end) = content.sel().unwrap();

            let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            content.reset_sel();
            *state = State::Insert;
        }
        Event::Key(Key::Ctrl('x')) => {
            let (beg, end) = content.sel().unwrap();

            let selection: String = content.iter().skip(beg).take(end - beg + 1).collect();
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(selection).unwrap();

            delete_sel(content);
            view.adjust_view(content.line());

            content.reset_sel();
            *state = State::Insert;
        }
        Event::Key(Key::Backspace) |
        Event::Key(Key::Delete) => {
            delete_sel(content);
            view.adjust_view(content.line());
            content.reset_sel();
            *state = State::Insert;
        }
        _ => {
            content.reset_sel();
            *state = State::Insert;

            treat_insert_event(content, view, event, state)
        }
    }
}

fn delete_sel<T>(content: &mut T)
    where T: Selectable + Editable
{
    let (beg, end) = content.sel().unwrap();
    let end = cmp::min(end + 1, content.len() - 1);
    content.move_to(end);
    for _ in beg..end {
        content.delete();
    }
}


pub fn treat_select_event<T>(content: &mut T, view: &mut View, event: Event, state: &mut State)
    where T: Editable + Selectable
{
    match event {
        Event::Mouse(MouseEvent::Hold(x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            if let State::Select(origin) = *state {
                let sel = (cmp::min(origin, content.pos()), cmp::max(origin, content.pos()));
                content.set_sel(sel);
            } else {
                panic!("Treating select event when event is not a Select");
            }
        }
        Event::Mouse(MouseEvent::Release(x, y)) => {
            let (line, col) = view.translate_coordinates(content, x, y);
            content.move_at(line, col);
            if let State::Select(origin) = *state {
                if origin != content.pos() {
                    let sel = (cmp::min(origin, content.pos()), cmp::max(origin, content.pos()));
                    content.set_sel(sel);
                    *state = State::Selected;
                } else {
                    *state = State::Insert;
                }
            } else {
                panic!("Treating select event when event is not a Select");
            }

        }
        _ => {}
    }
}
