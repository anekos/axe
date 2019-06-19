use std::fs::File;
use std::io;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc;
#[cfg(feature = "notification")]
use libnotify::{Notification, Urgency};
use patrol::Target;

use crate::args;
use crate::display;
use crate::errors::{AppError, AppResult, AppResultU};
use crate::types::*;



type Pid = Arc<Mutex<Option<u32>>>;


pub fn start() -> AppResultU {
    #[cfg(feature = "notification")]
    libnotify::init("axe").map_err(|_| AppError::Libnotify)?;

    let app_options = args::parse()?;

    let (tx, rx) = channel();

    let targets: Vec<Target<String>> = app_options.targets.to_vec();
    thread::spawn(move || {
        patrol::start(&targets, &tx);
    });

    let pid = Arc::new(Mutex::<Option<u32>>::new(None));
    let mut changed: Option<String> = None;

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

        if let Some((mut command, program)) = make_command(&app_options, changed.take())? {
            let t = Instant::now();

            if app_options.sync {
                match command.spawn() {
                    Ok(mut child) => on_exit(child.wait(), t, &program, None),
                    Err(err) => display::error(&format!("{}", err))
                }
            } else {
                let pid = pid.clone();
                thread::spawn(move || {
                    match command.spawn() {
                        Ok(mut child) => {
                            {
                                let mut pid = pid.lock().unwrap();
                                *pid = Some(child.id());
                            }
                            on_exit(child.wait(), t, &program, Some(pid.clone()));
                            let _ = pid.lock().unwrap().take();
                        },
                        Err(err) => display::error(&format!("{}", err))
                    }
                });
            }

            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            }
        }

        changed = Some(rx.recv().unwrap().data);
    }
}

#[cfg(feature = "notification")]
fn notify(message: &str) {
    let n = Notification::new("axe", Some(message), None);
    n.set_urgency(Urgency::Low);
    let _ = n.show();
}
#[cfg(not(feature = "notification"))]
fn notify(_: &str) {
}

fn on_exit(status: io::Result<ExitStatus>, at_start: Instant, program: &str, pid: Option<Pid>) {
    display::time(at_start.elapsed());
    match status {
        Ok(status) => match status.code() {
            Some(0) | None => {
                if let Some(pid) = pid {
                    if pid.lock().unwrap().is_none() {
                        return;
                    }
                }
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

fn concrete(cl: &[Part], changed: Option<String>, targets: &[Target<String>]) -> AppResult<Option<Vec<String>>> {
    cl.iter().map(|it| Ok(match it {
        Part::Literal(s) => Some(s.to_owned()),
        Part::Changed => changed.clone(),
        Part::Position(index) => if let Some(target) = targets.get(*index - 1) {
            Some(target.data.clone())
        } else {
            return Err(AppError::InvalidPosition(*index))
        }
    })).collect()
}

fn make_command(option: &AppOption, changed: Option<String>) -> AppResult<Option<(Command, String)>> {
    concrete(&option.command_line, changed, &option.targets)?.map(|command_line| {
        let (program, args) = command_line.split_first().ok_or(AppError::NotEnoughArguments)?;

        let mut command = Command::new(program);

        command.args(args);

        if let Some(stdin) = option.stdin.clone() {
            let stdin = File::open(stdin)?;
            command.stdin(stdin);
        }
        if let Some(stdout) = option.stdout.clone() {
            let stdout = File::open(stdout)?;
            command.stdout(stdout);
        }

        Ok((command, program.to_owned()))
    }).transpose()
}
