use super::{Movement, Editable, Saveable, Named, Undoable, Lines};
use std::io::Result;

pub type Selection = (usize, usize);

pub trait Selectable {
    fn sel(&self) -> &Option<Selection>;
    fn set_sel(&mut self, selection: Selection);
    fn reset_sel(&mut self);
    fn in_sel(&self, pos: usize) -> bool {
        match *self.sel() {
            Some((beg, end)) => pos >= beg && pos <= end,
            None => false,
        }
    }
    fn line_in_sel(&self, line: usize) -> bool;
    fn slice_sel(&self) -> String;
    fn delete_sel(&mut self);
}

pub struct Select<T>
    where T: Editable
{
    pub content: T,
    pub sel: Option<Selection>,
}

impl<T> Select<T>
    where T: Editable
{
    pub fn new(content: T) -> Select<T> {
        Select {
            content: content,
            sel: None,
        }
    }
}

impl<T> Selectable for Select<T>
    where T: Editable
{
    fn sel(&self) -> &Option<Selection> {
        &self.sel
    }

    fn set_sel(&mut self, selection: Selection) {
        self.sel = Some(selection);
    }

    fn reset_sel(&mut self) {
        self.sel = None;
    }

    fn line_in_sel(&self, line: usize) -> bool {
        self.in_sel(self.offset_of_line(line))
    }

    fn slice_sel(&self) -> String {
        if let Some((beg, end)) = self.sel {
            String::from(self.slice(beg, end).clone())
        } else {
            String::from("")
        }
    }
    fn delete_sel(&mut self) {
        if let Some((beg, end)) = self.sel {
            self.delete_range(beg, end);
        }
    }
}

impl<T> Editable for Select<T>
    where T: Editable
{
    delegate!{
        content:
            mut step(mov: Movement) -> (),
            mut move_to(pos: usize) -> (),
            mut move_at(line: usize, col: usize) -> (),
            mut insert(c: char) -> (),
            mut insert_forward(c: char) -> (),
            mut delete() -> Option<char>,
            mut delete_forward() -> Option<char>,
            mut delete_range(start: usize, end: usize) -> (),
            pos() -> usize,
            line() -> usize,
            col() -> usize,
            line_count() -> usize,
            len() -> usize,
            lines() -> Lines,
            slice(start: usize, end: usize) -> String,
            offset_of_line(line: usize) -> usize,
    }
}

impl<T> Saveable for Select<T>
    where T: Editable + Saveable
{
    delegate!{ content: save() -> Result<()> }
}

impl<T> Named for Select<T>
    where T: Editable + Named
{
    delegate!{ content: name() -> &String,
        mut set_name(name: String) -> (),
    }
}

impl<T> Undoable for Select<T>
    where T: Editable + Undoable
{
    delegate!{
        content:
            mut undo() -> (),
            mut redo() -> (),
    }
}
