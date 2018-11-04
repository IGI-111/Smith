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
use std::path::Path;
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
    let ps = SyntaxSet::load_defaults_newlines();
    let ts: Theme = from_binary(include_bytes!("../assets/gruvbox.themedump"));
    let mut text = build_text(&filename);
    let mut view = build_view(&filename, &ps, &ts);
    let mut state = State::Insert;

    let stdin = stdin();

    view.render(&text);

    let mut events = stdin.events();
    loop {
        if let Some(event) = events.next() {
            state = if let Some(next_state) = state.handle(&mut text, &mut view, event.unwrap()) {
                next_state
            } else {
                break;
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

fn build_view<'a>(filename: &Option<String>, ps: &SyntaxSet, theme: &'a Theme) -> View<'a> {
    match filename {
        Some(name) => {
            let extension = match Path::new(&name).extension() {
                Some(osstr) => match osstr.to_str() {
                    Some(s) => s,
                    None => return View::new(theme),
                },
                None => return View::new(theme),
            };

            let syntax = match ps.find_syntax_by_extension(extension) {
                Some(syn) => syn,
                None => return View::new(theme),
            };
            View::with_syntax(syntax, theme)
        }
        None => View::new(theme),
    }
}
