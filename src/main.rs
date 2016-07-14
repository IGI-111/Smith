extern crate rustbox;

#[macro_use]
mod macros;

mod view;
mod state;
mod command;

use std::default::Default;
use std::env;
use rustbox::RustBox;
use state::{Text, Recorded};
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
    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => v,
        Err(e) => panic!(e.to_string()),
    };
    let mut text = Recorded::new(match filename {
        Some(name) => {
            match Text::open_file(name) {
                Ok(v) => v,
                Err(e) => panic!(e.to_string()),
            }
        }
        None => Text::empty(),
    });
    let view = View::new(&rustbox);

    rustbox.clear();
    view.render(&mut text);

    loop {
        match rustbox.poll_event(false) {
            Ok(event) => {
                rustbox.clear();
                if command::treat_event(&mut text, &view, &event) {
                    break;
                }
                view.render(&mut text);
            }
            Err(e) => panic!(e.to_string()),
        }
    }
}
