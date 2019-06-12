
use std::process::exit;

mod app;
mod args;
mod display;
mod errors;
mod types;


const USAGE: &str = include_str!("usage.txt");

fn main() {
    match app::start() {
        Ok(_) | Err(errors::AppError::Libnotify) => (),
        Err(err) => {
            eprintln!("{}\n", err);
            eprint!("{}", USAGE);
            exit(1);
        },
    }
}
