#![allow(unknown_lints)]
#![allow(absurd_extreme_comparisons)]

mod action;

use std::collections::VecDeque;
use std::io::Result;
use super::{Movement, Editable, Saveable, Named, Lines};
use self::action::Action;
use std::usize;

const HISTORY_SIZE: usize = usize::MAX;
const UNDO_SIZE: usize = usize::MAX;

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
        let from = self.content.pos();
        self.content.step(mov);
        let to = self.content.pos();
        self.record(Action::Move(to as isize - from as isize));
    }

    fn move_to(&mut self, pos: usize) {
        let from = self.content.pos();
        self.content.move_to(pos);
        let to = self.content.pos();
        self.record(Action::Move(to as isize - from as isize));
    }

    fn move_at(&mut self, line: usize, col: usize) {
        let from = self.content.pos();
        self.content.move_at(line, col);
        let to = self.content.pos();
        self.record(Action::Move(to as isize - from as isize));
    }

    fn insert(&mut self, c: char) {
        let mut s = String::new();
        s.push(c);
        self.record(Action::Insert(s));
        self.content.insert(c);
    }

    fn insert_forward(&mut self, c: char) {
        let mut s = String::new();
        s.push(c);
        self.record(Action::InsertForward(s));
        self.content.insert_forward(c);
    }

    fn delete(&mut self) -> Option<char> {
        let c = self.content.delete();
        if let Some(c) = c {
            let mut s = String::new();
            s.push(c);
            self.record(Action::Delete(s))
        }
        c
    }

    fn delete_forward(&mut self) -> Option<char> {
        let c = self.content.delete_forward();
        if let Some(c) = c {
            let mut s = String::new();
            s.push(c);
            self.record(Action::DeleteForward(s))
        }
        c
    }

    fn delete_range(&mut self, start: usize, end: usize) {
        let text = self.slice(start, end);
        let pos = self.pos();
        self.content.delete_range(start, end);
        self.record(Action::Move(end as isize - pos as isize));
        self.record(Action::Delete(text));
        self.record(Action::Move(pos as isize - start as isize));
    }

    delegate!{
        content:
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

impl<T> Saveable for Recorded<T>
    where T: Editable + Saveable
{
    delegate!{ content: save() -> Result<()> }
}

impl<T> Named for Recorded<T>
    where T: Editable + Named
{
    delegate!{ content:
        name() -> &String,
        mut set_name(name: String) -> (),
    }
}
