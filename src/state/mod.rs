mod text;
mod record;
mod select;

pub use self::text::Text;
pub use self::record::Recorded;
pub use self::record::Undoable;
pub use self::select::Select;

use std::io::Result;
use ropey::Rope;

pub trait Editable {
    fn step(&mut self, mov: Movement);
    fn move_to(&mut self, pos: usize);
    fn insert(&mut self, c: char);
    fn delete(&mut self) -> Option<char>;
    fn pos(&self) -> usize;
    fn line(&self) -> usize;
    fn col(&self) -> usize;
    fn line_count(&self) -> usize;
    fn as_rope(&self) -> &Rope;

}

pub trait Named {
    fn name(&self) -> &String;
    fn set_name(&mut self, name: String);
}

pub trait Saveable: Named {
    fn save(&self) -> Result<()>;
}

#[derive(Clone)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    LineStart,
    LineEnd,
    PageUp(usize),
    PageDown(usize),
}
