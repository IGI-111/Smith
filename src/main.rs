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
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock, mpsc};

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

    let text_handle_1 = Arc::new(RwLock::new(build_text(filename)));
    let text_handle_2 = text_handle_1.clone();

    let view_handle_1 = Arc::new(RwLock::new(View::new().unwrap()));
    let view_handle_2 = view_handle_1.clone();

    let mut command = Command::new();

    let stdin = stdin();

    let (tx, rx) = mpsc::channel();

    let event_thread = thread::spawn(move || {
        let mut events = stdin.events();

        let mut is_exit = false;
        while !is_exit {
            let event = events.next().unwrap();

            {
                let ref mut text = *text_handle_1.write().unwrap();
                let ref mut view = *view_handle_1.write().unwrap();
                is_exit = command.treat_event(text, view, event.unwrap());
            }
            tx.send(is_exit);
        }
    });
    let render_thread = thread::spawn(move || {
        {
            // render once first
            let ref text = *text_handle_2.read().unwrap();
            let ref view = *view_handle_2.read().unwrap();
            view.render(text).unwrap();
        }

        let now = Instant::now();
        while !rx.recv().unwrap() {
            let min_period = Duration::from_millis(16);
            let elapsed = now.elapsed();

            {
                let ref view = *view_handle_2.read().unwrap();
                let ref text = *text_handle_2.read().unwrap();
                view.render(text).unwrap();
            }

            if elapsed <= min_period {
                thread::sleep(min_period-elapsed);
            }
        }
    });
    event_thread.join();
    render_thread.join();
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
