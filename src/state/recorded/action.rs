use state::{Editable, Position};

#[derive(Clone, Debug)]
pub enum Action {
    Insert(String),
    Delete(String),
    Move {
        from: Position,
        to: Position,
    },
}

impl Eq for Action {}
impl PartialEq for Action {
    fn eq(&self, other: &Action) -> bool {
        match self {
            &Action::Insert(_) => {
                match other {
                    &Action::Insert(_) => true,
                    _ => false,
                }
            }
            &Action::Delete(_) => {
                match other {
                    &Action::Delete(_) => true,
                    _ => false,
                }
            }
            &Action::Move { from: _, to: _ } => {
                match other {
                    &Action::Move { from: _, to: _ } => true,
                    _ => false,
                }
            }
        }
    }
}

impl Action {
    pub fn apply<T: Editable>(&self, content: &mut T) {
        match self {
            &Action::Insert(ref s) => {
                for c in s.chars() {
                    content.insert(c);
                }
            }
            &Action::Delete(ref s) => {
                for _ in s.chars() {
                    content.delete();
                }
            }
            &Action::Move { from: _, ref to } => {
                content.move_to(to.clone());
            }
        };
    }

    pub fn invert(&self) -> Action {
        match self {
            &Action::Insert(ref s) => Action::Delete(s.clone()),
            &Action::Delete(ref s) => Action::Insert(s.clone()),
            &Action::Move { ref from, ref to } => {
                Action::Move {
                    from: to.clone(),
                    to: from.clone(),
                }
            }
        }
    }

    pub fn join(&mut self, act: Action) {
        assert_eq!(act, *self);
        match self {
            &mut Action::Insert(ref mut s) => {
                let ref act_string = match act {
                    Action::Insert(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(act_string)
            }
            &mut Action::Delete(ref mut s) => {
                let ref act_string = match act {
                    Action::Delete(a) => a,
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                s.push_str(act_string)
            }
            &mut Action::Move { from: _, ref mut to } => {
                let act_to = match act {
                    Action::Move { from: _, to: val } => val.clone(),
                    _ => panic!("Trying to join dissimilar Actions"),
                };
                *to = act_to;
            }
        }
    }
}
