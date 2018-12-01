#![recursion_limit = "256"]

extern crate clipboard;
extern crate ndarray;
extern crate ropey;
extern crate termion;
extern crate unicode_segmentation;
extern crate unicode_width;
#[macro_use]
extern crate delegate;
extern crate syntect;

mod command;
mod data;
mod view;

use command::State;
use data::{Recorded, Select, Text};
use std::env;
use std::io::stdin;
use syntect::dumps::from_binary;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use termion::input::TermRead;
use view::View;

fn main() {
    let args = env::args();

    if args.len() > 1 {
        for filename in args.skip(1) {
            edit_file(&Some(filename));
        }
    } else {
        edit_file(&None);
    }
}

fn edit_file(filename: &Option<String>) {
    let ps = SyntaxSet::load_defaults_nonewlines();
    let ts: Theme = from_binary(include_bytes!("../assets/gruvbox.themedump"));
    let mut text = build_text(&filename);
    let mut view = build_view(&filename, &ps, &ts);
    let mut state = State::Insert;

    let stdin = stdin();

    view.render(&text);

    let mut events = stdin.events();
    loop {
        if let Some(event) = events.next() {
            state = match state.handle(&mut text, &mut view, event.unwrap()) {
                State::Exit => break,
                State::Open(new_filename) => {
                    // we must close the terminal modes before resetting them
                    drop(text);
                    drop(view);
                    text = build_text(&Some(new_filename.clone()));
                    view = build_view(&Some(new_filename.clone()), &ps, &ts);
                    view.message(&format!("Opened {}", new_filename));
                    State::Insert
                }
                state => state,
            }
        }

        view.render(&text);
    }
}

fn build_text(filename: &Option<String>) -> Select<Recorded<Text>> {
    Select::new(Recorded::new(match filename {
        Some(name) => match Text::open_file(name.clone()) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        },
        None => Text::empty(),
    }))
}

fn build_view<'a>(filename: &Option<String>, ps: &'a SyntaxSet, theme: &'a Theme) -> View<'a> {
    let syntax = match filename {
        Some(filename) => match ps.find_syntax_for_file(filename) {
            Ok(Some(syn)) => syn,
            _ => ps.find_syntax_plain_text(),
        },
        None => ps.find_syntax_plain_text(),
    };

    View::new(theme, syntax, ps)
}
