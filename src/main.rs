extern crate termion;
extern crate clipboard;

#[macro_use]
mod macros;

mod view;
mod state;
mod command;

use std::env;
use state::{Text, Recorded, Select};
use view::View;
use termion::{TermRead, async_stdin};
use command::Command;

fn main() {
    let args = env::args();
    if args.len() > 1 {
        for filename in args.skip(1) {
            edit_file(Some(filename));
        }
    } else {
        edit_file(None);
    }
}

fn edit_file(filename: Option<String>) {
    let ref mut text = build_text(filename);
    let ref mut view = View::new().unwrap();
    let mut command = Command::new();

    view.render(text).unwrap();

    let mut stdin = async_stdin().keys();
    loop {
        match stdin.next() {
            Some(key) => {
                if command.treat_event(text, view, key.unwrap()) { break; }
                view.render(text).unwrap();
            },
            None => {},
        }
    }
}

fn build_text(filename: Option<String>) -> Recorded<Select<Text>> {
    Recorded::new(Select::new(match filename {
        Some(name) => {
            match Text::open_file(name) {
                Ok(v) => v,
                Err(e) => panic!(e.to_string()),
            }
        }
        None => Text::empty(),
    }))
}
