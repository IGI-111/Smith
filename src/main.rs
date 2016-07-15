extern crate termion;
extern crate clipboard;

#[macro_use]
mod macros;

mod view;
mod state;
mod command;

use std::default::Default;
use std::env;
use std::panic;
use state::{Text, Recorded, Select};
use view::View;
use std::io::stdin;
use termion::TermRead;
use command::treat_event;

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
    let ref mut view = View::new();

    view.render(text);

    let stdin = stdin().keys();
    for key in stdin {
        if treat_event(text, view, key.unwrap()) { break; }
        view.render(text);
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
