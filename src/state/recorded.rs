use std::collections::VecDeque;
use std::io::Result;
use super::{Position, Movement, Editable, Saveable, Named};

const HISTORY_SIZE: usize = 10;
const UNDO_SIZE: usize = 5;

// pub trait Undoable {
//     fn undo(&mut self);
//     fn redo(&mut self);
// }

pub struct Recorded<T> where T: Editable {
    pub content: T ,
    // history: VecDeque<Action>,
    // undone: VecDeque<Action>,
}

impl<T> Recorded<T> where T: Editable {
    pub fn new(content: T) -> Recorded<T> {
        Recorded{
            content: content,
            // history: VecDeque::new(),
            // undone: VecDeque::new(),
        }
    }
    // fn record(&mut self, act: Action) {
    //     self.history.push_back(act);
    //     if self.history.len() > HISTORY_SIZE {
    //         self.history.pop_front();
    //     }
    // }
}

// impl<T> Undoable for Recorded<T> where T:Editable {
//     pub fn undo(&mut self) {
//         let to_undo = history.pop_back();
//         //undo
//         self.undone.push_back(to_undo);
//         if self.undone.len() > UNDO_SIZE {
//             self.undone.pop_front();
//         }
//     }
//     pub fn redo(&mut self) {

//     }
// }

impl<T> Saveable for Recorded<T> where T: Editable + Saveable {
    fn save(&self) -> Result<()> {
        self.content.save()
    }
}

impl<T> Named for Recorded<T> where T: Editable + Named {
    fn name(&self) -> &String {
        self.content.name()
    }
}

impl<T> Editable for Recorded<T> where T: Editable {
    fn step(&mut self, mov: Movement) {
        self.content.step(mov);
    }
    fn insert(&mut self, c: char) {
        // self.record(Action::Insert(c));
        self.content.insert(c);
    }
    fn delete(&mut self) -> Option<char> {
        let c = self.content.delete();
        // match c {
        //     Some(c) => self.record(Action::Delete(c)),
        //     None => {}
        // }
        c
    }
    fn pos(&self) -> &Position { self.content.pos() }

    fn lines(&self) -> &Vec<String> { self.content.lines() }
}

// enum Action {
//     Insert(Vec<char>),
//     Delete(Vec<char>),
// }

// fn apply_action<T:Editable>(act: Action, content: &mut T){
//     match act {
//         Insert(c) => content.insert(c),
//         Delete(c) => content.delete(),
//     }
// }

// fn invert_action(act: Action) -> Action {
//     match act {
//         Insert(chars) => Delete(c),
//         Delete(c) => Insert(c),
//     }
// }
