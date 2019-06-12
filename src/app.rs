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
    let app_options = args::parse()?;

    let (program, args) = app_options.command_line.split_first().ok_or(AppError::NotEnoughArguments)?;

    let (tx, rx) = channel();

    let targets = app_options.targets;
    thread::spawn(move || {
        patrol::start(targets, tx);
    });

    let pid = Arc::new(Mutex::<Option<u32>>::new(None));

    loop {
        if let Some(pid) = (*pid.lock().unwrap()).take() {
            display::killing(pid);
            unsafe {
                let mut status = 1;
                libc::kill(pid as i32, app_options.signal);
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

        let _ = rx.recv().unwrap();
    }
}
