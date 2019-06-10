
use std::process::exit;

mod app;
mod args;
mod display;
mod errors;
mod types;


const USAGE: &str = include_str!("usage.txt");

fn main() {
    if let Err(err) = app::start() {
        eprintln!("{}\n", err);
        eprint!("{}", USAGE);
        exit(1);
    }
}
