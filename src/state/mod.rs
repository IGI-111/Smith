mod text;
mod recorded;

pub use self::text::Text;
pub use self::recorded::Recorded;

use std::clone::Clone;
use std::io::Result;

pub trait Editable {
    fn step(&mut self, mov: Movement);
    fn insert(&mut self, c: char);
    fn delete(&mut self) -> Option<char>;
    fn pos(&self) -> &Position;
    fn lines(&self) -> &Vec<String>;
}

pub trait Named {
    fn name(&self) -> &String;
}

pub trait Saveable : Named {
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
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Position {
        Position {
            line: line,
            column: column,
        }
    }
}
