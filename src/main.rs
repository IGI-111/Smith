#![recursion_limit = "256"]

extern crate clipboard;
extern crate ropey;
extern crate termion;
extern crate unicode_segmentation;
extern crate unicode_width;
#[macro_use]
extern crate delegate;

mod command;
mod data;
mod view;

use command::State;
use data::{Recorded, Select, Text};
use std::env;
use std::io::stdin;
use termion::input::TermRead;
use view::View;

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
    let mut view = View::new();
    let mut state = State::Insert;

    let stdin = stdin();

    view.render(&text);

    let mut events = stdin.events();
    loop {
        if let Some(event) = events.next() {
            state = match state.handle(&mut text, &mut view, event.unwrap()) {
                Some(state) => state,
                None => break,
            }
        }
        view.render(&text);
    }
}

fn build_text(filename: Option<String>) -> Select<Recorded<Text>> {
    Select::new(Recorded::new(match filename {
        Some(name) => match Text::open_file(name) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        },
        None => Text::empty(),
    }))
}
