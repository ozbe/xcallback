#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate objc;

mod cli;
mod macos;

use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        cli::run(receiver, &macos::open);
        macos::terminate_app();
    });
    macos::run_app(sender);
}
