use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc;

use crate::args;
use crate::display;
use crate::errors::{AppError, AppResultU};



pub fn start() -> AppResultU {
    let (targets, command) = args::parse(env::args().skip(1).map(to_absolute_path).collect())?;

    let (program, args) = command.split_first().ok_or(AppError::NotEnoughArguments)?;

    let (tx, rx) = channel();

    thread::spawn(move || {
        patrol::start(targets, tx);
    });

    let pid = Arc::new(Mutex::<Option<u32>>::new(None));

    loop {
        let _ = rx.recv().unwrap();

        if let Some(pid) = (*pid.lock().unwrap()).take() {
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
                    {
                        let mut pid = pid.lock().unwrap();
                        *pid = None;
                    }
                    display::time(t.elapsed());
                },
                Err(err) => display::error(&format!("{}", err))
            }
        });

        while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
        }
    }
}

fn to_absolute_path(path: String) -> String {
    let buf = PathBuf::from(&path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or_else(|_| path.to_owned())
}
