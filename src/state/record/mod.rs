mod action;

use std::collections::VecDeque;
use std::io::Result;
use super::{Position, Movement, Editable, Saveable, Named};
use self::action::Action;

const HISTORY_SIZE: usize = 100;
const UNDO_SIZE: usize = 100;

pub trait Undoable {
    fn undo(&mut self);
    fn redo(&mut self);
}

pub struct Recorded<T>
where T: Editable
{
    pub content: T,
    pub history: VecDeque<Action>,
    pub undone: VecDeque<Action>,
}

impl<T> Recorded<T>
where T: Editable
{
    pub fn new(content: T) -> Recorded<T> {
        Recorded {
            content: content,
            history: VecDeque::new(),
            undone: VecDeque::new(),
        }
    }
    fn record(&mut self, act: Action) {
        self.undone.clear(); // we are branching to a new sequence of events
        if let Some(a) = self.history.front_mut() {
            if *a == act {
                // join similar actions together
                a.join(act);
                return;
            }
        }
        self.history.push_front(act);
        while self.history.len() > HISTORY_SIZE {
            self.history.pop_back();
        }
    }
}

impl<T> Undoable for Recorded<T>
where T: Editable
{
    fn undo(&mut self) {
        let to_undo = match self.history.pop_front() {
            None => return,
            Some(a) => a,
        };
        self.undone.push_front(to_undo.clone());
        while self.undone.len() > UNDO_SIZE {
            self.undone.pop_back();
        }
        to_undo.invert().apply(&mut self.content);

    }
    fn redo(&mut self) {
        let to_redo = match self.undone.pop_front() {
            None => return,
            Some(a) => a,
        };
        to_redo.apply(&mut self.content);
        self.history.push_front(to_redo);
    }
}

impl<T> Editable for Recorded<T>
where T: Editable
{
    fn step(&mut self, mov: Movement) {
        let from = self.content.pos().clone();
        self.content.step(mov);
        let to = self.content.pos().clone();
        self.record(Action::Move {
            from: from,
            to: to,
        });
    }

    fn move_to(&mut self, pos: Position) {
        let from = self.content.pos().clone();
        self.content.move_to(pos);
        let to = self.content.pos().clone();
        self.record(Action::Move {
            from: from,
            to: to,
        });
    }

    fn insert(&mut self, c: char) {
        let mut s = String::new();
        s.push(c);
        self.record(Action::Insert(s));
        self.content.insert(c);
    }

    fn delete(&mut self) -> Option<char> {
        let c = self.content.delete();
        match c {
            Some(c) => {
                let mut s = String::new();
                s.push(c);
                self.record(Action::Delete(s))
            }
            None => {}
        }
        c
    }

    delegate!{
        content:
            pos() -> &Position,
            lines() -> &Vec<String>,
    }
}

impl<T> Saveable for Recorded<T>
where T: Editable + Saveable
{
    delegate!{ content: save() -> Result<()> }
}

impl<T> Named for Recorded<T>
where T: Editable + Named
{
    delegate!{ content: name() -> &String }
}
