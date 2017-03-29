
extern crate ansi_term;
extern crate chrono;
extern crate patrol;

mod display;

use std::env::args;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::mpsc::channel;
use std::thread;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use patrol::Target;



const SEP: &'static str = "--";
const PLACE_HOLDER: &'static str = "%";

type Parsed = (Vec<Target>, Vec<String>);


fn parse_arguments(a: Vec<String>) -> Result<Parsed, String> {
    fn sep(it: &String) -> bool { it == SEP }
    fn target(it: &String) -> Target { Target::new(it) }

    let (targets, mut command) = if let Some(at) = a.iter().position(sep) {
        let (lefts, rights) = a.split_at(at);
        (lefts.to_vec(), rights[1..].to_vec())
    } else if let Some((head, tail)) = a.split_first() {
        if tail.len() == 0 {
            (vec![head.to_owned()], vec![head.to_owned()])
        } else {
            (vec![head.to_owned()], tail.to_vec())
        }
    } else {
        return Err("Not enought arguments".to_owned())
    };

    if let Some(first) = targets.first() {
        for it in command.as_mut_slice().iter_mut() {
            if *it == PLACE_HOLDER {
                *it = first.clone()
            }
        }
    }

    // println!("targets: {:?}", targets);
    // println!("command: {:?}", command);

    for it in &targets {
        if !Path::new(it).exists() {
            return Err(format!("Target not found: {}", it));
        }
    }

    Ok((targets.iter().map(target).collect(), command))
}


fn die(message: &str) {
    println!("{}\n", message);

    println!("Usage: axe <WATCH_TARGET> ... \"--\" <COMMAND_LINE> ...");
    println!("       axe <WATCH_TARGET> <COMMAND_LINE> ...");
    println!("       axe <WATCH_TARGET_AND_COMMAND>");
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
                    patrol::start(targets, tx);
                });

                loop {
                    let _ = rx.recv().unwrap();
                    display::separator();

                    thread::sleep(Duration::from_millis(100));

                    let t = Instant::now();

                    match Command::new(program).args(args.as_slice()).spawn() {
                        Ok(mut child) => {
                            child.wait().unwrap();
                            display::time(t.elapsed());
                        },
                        Err(err) => display::error(&format!("{}", err))
                    }

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
