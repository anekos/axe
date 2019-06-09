
mod display;
mod errors;

use std::env::args;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc;
use patrol::Target;

use errors::{AppError, AppResult, AppResultU};



const SEP: &str = "--";
const PLACE_HOLDER: &str = "%";

type Parsed = (Vec<Target>, Vec<String>);


fn extract_params(a: Vec<String>) -> AppResult<(Vec<String>, Vec<String>)> {
    fn sep(it: &str) -> bool { it == SEP }

    if let Some(at) = a.iter().map(String::as_ref).position(sep) {
        let (lefts, rights) = a.split_at(at);
        return Ok((lefts.to_vec(), rights[1..].to_vec()));
    }

    if let Some((head, tail)) = a.split_first() {
        return Ok(if tail.is_empty() {
            (vec![head.to_owned()], vec![head.to_owned()])
        } else {
            (vec![head.to_owned()], tail.to_vec())
        });
    }

    Err(AppError::NotEnoughArguments)
}


fn parse_arguments(a: Vec<String>) -> AppResult<Parsed> {
    fn target(it: &str) -> Target { Target::new(it) }

    let (targets, mut command) = extract_params(a)?;

    if let Some(first) = targets.first() {
        for it in command.as_mut_slice().iter_mut() {
            if *it == PLACE_HOLDER {
                *it = first.clone()
            }
        }
    }

    for it in &targets {
        if !Path::new(it).exists() {
            return Err(AppError::TargetNotFound(it.to_owned()));
        }
    }

    Ok((targets.iter().map(String::as_ref).map(target).collect(), command))
}


fn app() -> AppResultU {
    let (targets, command) = parse_arguments(args().skip(1).map(to_absolute_path).collect())?;

    let (program, args) = command.split_first().ok_or(AppError::NotEnoughArguments)?;

    let (tx, rx) = channel();

    thread::spawn(move || {
        patrol::start(targets, tx);
    });

    let pid = Arc::new(Mutex::<Option<u32>>::new(None));

    loop {
        let _ = rx.recv().unwrap();

        if let Some(pid) = *pid.lock().unwrap() {
            display::killing(pid);
            unsafe {
                let mut status = 1;
                libc::kill(pid as i32, libc::SIGTERM);
                libc::waitpid(pid as i32, &mut status, 0);
            };
        }

        display::separator();

        thread::sleep(Duration::from_millis(100));

        let (program, args) = (program.to_owned(), args.to_owned());
        let pid = pid.clone();

        thread::spawn(move || {
            let t = Instant::now();
            match Command::new(program).args(args).spawn() {
                Ok(mut child) => {
                    {
                        let mut pid = pid.lock().unwrap();
                        *pid = Some(child.id());
                    }
                    let _ = child.wait();
                    display::time(t.elapsed());
                },
                Err(err) => display::error(&format!("{}", err))
            }
        });

        while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
        }
    }
}


fn main() {
    if let Err(err) = app() {
        eprintln!("{}\n", err);
        eprintln!("Usage: axe <WATCH_TARGET> ... \"--\" <COMMAND_LINE> ...");
        eprintln!("       axe <WATCH_TARGET> <COMMAND_LINE> ...");
        eprintln!("       axe <WATCH_TARGET_AND_COMMAND>");
        exit(1);
    }
}


fn to_absolute_path(path: String) -> String {
    let buf = PathBuf::from(&path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or_else(|_| path.to_owned())
}
