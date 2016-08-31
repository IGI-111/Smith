use super::{Movement, Editable, Saveable, Named, Undoable};
use std::io::Result;
use ropey::Rope;

pub type Selection = (usize, usize);

pub trait Selectable {
    fn selection(&self) -> &(usize, usize);
    fn select(&mut self, (usize, usize));
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

impl<T> Editable for Select<T>
    where T: Editable
{
    delegate!{
        content:
            mut step(mov: Movement) -> (),
            mut move_to(pos: usize) -> (),
            mut insert(c: char) -> (),
            mut delete() -> Option<char>,
            pos() -> usize,
            line() -> usize,
            col() -> usize,
            line_count() -> usize,
            as_rope() -> &Rope,
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
