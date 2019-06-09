
use std::process::exit;

mod app;
mod args;
mod display;
mod errors;
mod types;



fn main() {
    if let Err(err) = app::start() {
        eprintln!("{}\n", err);
        eprintln!("Usage: axe <WATCH_TARGET> ... \"--\" <COMMAND_LINE> ...");
        eprintln!("       axe <WATCH_TARGET> <COMMAND_LINE> ...");
        eprintln!("       axe <WATCH_TARGET_AND_COMMAND>");
        exit(1);
    }
}
