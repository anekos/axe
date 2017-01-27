
extern crate ansi_term;
extern crate chrono;
extern crate inotify;


mod notifier;
mod display;


use std::env::args;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::mpsc::channel;
use std::thread;
use std::path::PathBuf;
use std::process::{Command, exit};
use notifier::Target;


const SEP: &'static str = "--";


fn parse_arguments(a: Vec<String>) -> (Vec<Target>, Vec<String>) {
    fn sep(it: &String) -> bool { it == SEP }
    fn nsep(it: &String) -> bool { it != SEP }
    fn target(it: String) -> Target { Target::new(&it) }

    let b = a.clone();

    if a.iter().skip(1).any(sep) {
        (
            a.into_iter().take_while(nsep).map(target).collect(),
            b.into_iter().skip_while(nsep).skip(1).collect()
        )
    } else {
        (
            a.into_iter().take(1).map(target).collect(),
            b.into_iter().skip(1).collect()
        )
    }
}


fn main() {
    if args().count() == 1 {
        println!("Usage: axe <WATCH_TARGET> ... \"--\" <COMMAND_LINE> ...");
        println!("       axe <WATCH_TARGET> <COMMAND_LINE> ...");
        exit(1);
    }

    let (targets, command) = parse_arguments(args().skip(1).map(to_absolute_path).collect());

    // println!("targets: {:?}", targets);
    // println!("command: {:?}", command);

    if let Some(program) = command.first() {
        let args: Vec<&String> = command.iter().skip(1).collect();

        let (tx, rx) = channel();

        thread::spawn(move || {
            notifier::start_to_watch(targets, tx);
        });

        loop {
            let _ = rx.recv().unwrap();
            display::separator();

            let t = Instant::now();

            let mut child = Command::new(program).args(args.as_slice()).spawn().unwrap();
            child.wait().unwrap();

            display::time(t.elapsed());

            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            }
        }
    };
}


fn to_absolute_path(path: String) -> String {
    let buf = PathBuf::from(&path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or(path.to_owned())
}
