use super::{Position, Movement, Editable, Saveable, Named, Undoable};
use std::io::Result;

pub type Selection = (Position, Position);

pub trait Selectable {
    fn selection(&self) -> &(Position, Position);
    fn select(&mut self, (Position, Position));
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
            mut move_to(pos: Position) -> (),
            mut insert(c: char) -> (),
            mut delete() -> Option<char>,
            pos() -> &Position,
            lines() -> &Vec<String>,
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
    delegate!{ content: name() -> &String }
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
