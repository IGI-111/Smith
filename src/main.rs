extern crate termion;
extern crate clipboard;
extern crate ropey;

#[macro_use]
mod macros;

mod view;
mod state;
mod command;

use std::env;
use state::{Text, Recorded, Select};
use view::View;
use std::io::stdin;
use termion::input::TermRead;
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

    let mut text = build_text(filename);
    let mut view = View::new().unwrap();
    let mut command = Command::new();

    let stdin = stdin();

    view.render(&text).unwrap();

    let mut events = stdin.events();
    loop {
        if let Some(event) = events.next() {
            if command.treat_event(&mut text, &mut view, event.unwrap()) {
                break;
            }
        }
        view.render(&text).unwrap();
    }
}

fn build_text(filename: Option<String>) -> Select<Recorded<Text>> {
    Select::new(Recorded::new(match filename {
        Some(name) => {
            match Text::open_file(name) {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            }
        }
        None => Text::empty(),
    }))
}
