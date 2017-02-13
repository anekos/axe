
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

type Parsed = (Vec<Target>, Vec<String>);


fn parse_arguments(a: Vec<String>) -> Result<Parsed, String> {
    fn sep(it: &String) -> bool { it == SEP }
    fn nsep(it: &String) -> bool { it != SEP }
    fn target(it: String) -> Target { Target::new(&it) }

    let b = a.clone();

    let (targets, command): Parsed = if a.iter().skip(1).any(sep) {
        (
            a.into_iter().take_while(nsep).map(target).collect(),
            b.into_iter().skip_while(nsep).skip(1).collect()
        )
    } else {
        (
            a.into_iter().take(1).map(target).collect(),
            b.into_iter().skip(1).collect()
        )
    };

    if let Some(not_found) = targets.iter().cloned().find(|it| !it.exists()) {
        Err(format!("Target does not exist: {}", not_found.path.to_str().unwrap()).to_string())
    } else {
        Ok((targets, command))
    }
}


fn die(message: &str) {
    println!("{}\n", message);

    println!("Usage: axe <WATCH_TARGET> ... \"--\" <COMMAND_LINE> ...");
    println!("       axe <WATCH_TARGET> <COMMAND_LINE> ...");
    exit(1);
}


fn main() {
    match parse_arguments(args().skip(1).map(to_absolute_path).collect()) {
        Err(err) => {
            die(&format!("Error: {}", err))
        },
        Ok((targets, command)) => {
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
            } else {
                die("Error: Not enough arguments")
            }
        }
    }
}


fn to_absolute_path(path: String) -> String {
    let buf = PathBuf::from(&path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or(path.to_owned())
}
