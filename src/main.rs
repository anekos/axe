
use std::process::exit;
use std::thread;

use enclose::enclose;
#[cfg(feature = "notification")]
use libnotify;
use signal_hook::iterator::Signals;
use signal_hook::{SIGTERM, SIGQUIT, SIGINT};

mod app;
mod args;
mod display;
mod errors;
mod process;
mod types;

use errors::AppResult;
use process::Process;



const USAGE: &str = include_str!("usage.txt");


fn main() {
    #[cfg(feature = "notification")]
    let _ = libnotify::init("axe");

    let app_options = wrap(args::parse());

    let signals = Signals::new(&[SIGTERM, SIGQUIT, SIGINT]).unwrap();
    let process = Process::new(app_options.signal);

    thread::spawn(enclose!((process) move || wrap(app::start(app_options, process))));

    while 0 == signals.wait().count() {
        // DO NOTHING
    }
    process.terminate().unwrap();
}

fn wrap<T>(result: AppResult<T>) -> T {
    match result {
        Ok(result) => result,
        Err(err) => {
            eprintln!("{}\n", err);
            eprint!("{}", USAGE);
            exit(1);
        },
    }
}
