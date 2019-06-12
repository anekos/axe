use std::io;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc;
use libnotify::{Notification, Urgency};

use crate::args;
use crate::display;
use crate::errors::{AppError, AppResultU};



pub fn start() -> AppResultU {
    libnotify::init("axe").map_err(|_| AppError::Libnotify)?;

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
            unsafe { let mut status = 1;
                libc::kill(pid as i32, app_options.signal);
                libc::waitpid(pid as i32, &mut status, 0);
            };
        }

        display::separator();

        thread::sleep(Duration::from_millis(100));

        let (program, args) = (program.to_owned(), args.to_owned());

        let t = Instant::now();

        if app_options.sync {
            match Command::new(program.clone()).args(args.as_slice()).spawn() {
                Ok(mut child) => on_exit(child.wait(), t, &program, true),
                Err(err) => display::error(&format!("{}", err))
            }
        } else {
            let pid = pid.clone();
            thread::spawn(move || {
                match Command::new(program.clone()).args(args).spawn() {
                    Ok(mut child) => {
                        {
                            let mut pid = pid.lock().unwrap();
                            *pid = Some(child.id());
                        }
                        on_exit(child.wait(), t, &program, false);
                        {
                            let mut pid = pid.lock().unwrap();
                            *pid = None;
                        }
                    },
                    Err(err) => display::error(&format!("{}", err))
                }
            });
        }

        while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
        }

        let _ = rx.recv().unwrap();
    }
}

fn notify(message: &str) {
    let n = Notification::new("axe", Some(message), None);
    n.set_urgency(Urgency::Low);
    let _ = n.show();
}

fn on_exit(status: io::Result<ExitStatus>, at_start: Instant, program: &str, sync: bool) {
    display::time(at_start.elapsed());
    match status {
        Ok(status) => match status.code() {
            Some(0) | None => if sync {
                notify(&format!("OK - {}", program));
            },
            Some(code) =>
                notify(&format!("[{}] - {}", code, program)),
        },
        Err(ref err) if err.raw_os_error() == Some(libc::ECHILD) =>
            (),
        Err(err) =>
            display::error(&format!("Failed: {} {:?} {:?}", err, err.kind(), err.raw_os_error())),

    }
}
