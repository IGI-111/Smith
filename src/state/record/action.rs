use state::Editable;

#[derive(Clone, Debug)]
pub enum Action {
    Insert(String),
    Delete(String),
    Move(isize),
}

impl PartialEq for Action {
    fn eq(&self, other: &Action) -> bool {
        match *self {
            Action::Insert(_) => {
                if let Action::Insert(_) = *other {
                    true
                } else {
                    false
                }
            }
            Action::Delete(_) => {
                if let Action::Delete(_) = *other {
                    true
                } else {
                    false
                }
            }
            Action::Move(_) => {
                if let Action::Move(_) = *other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Action {
    pub fn apply<T: Editable>(&self, content: &mut T) {
        match *self {
            Action::Insert(ref s) => {
                for c in s.chars() {
                    content.insert(c);
                }
            }
            Action::Delete(ref s) => {
                for _ in s.chars() {
                    content.delete();
                }
            }
            Action::Move(rel) => {
                let new_position = rel + content.pos() as isize;
                content.move_to(new_position as usize);
            }
        };
    }

    pub fn invert(&self) -> Action {
        match *self {
            Action::Insert(ref s) => Action::Delete(s.clone()),
            Action::Delete(ref s) => Action::Insert(s.clone()),
            Action::Move(ref rel) => Action::Move(-rel),
        }
    }

    pub fn join(&mut self, act: Action) {
        assert_eq!(act, *self);
        match *self {
            Action::Insert(ref mut s) => {
                let act_string = match act {
                    Action::Insert(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(&act_string)
            }
            Action::Delete(ref mut s) => {
                let act_string = match act {
                    Action::Delete(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(&act_string)
            }
            Action::Move(ref mut rel) => {
                let act_rel = match act {
                    Action::Move(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                *rel += act_rel;
            }
        }
    }
}
