#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate objc;

mod cli;
mod macos;
mod x_callback_url;

use std::sync::mpsc;
use std::thread;
use crate::macos::NSXCallbackClient;

fn main() {
    thread::spawn(move || {
        cli::run(&NSXCallbackClient::new());
        macos::terminate_app();
    });
    macos::run_app();
}
