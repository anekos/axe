use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc;
#[cfg(feature = "notification")]
use libnotify::{Notification, Urgency};
use patrol::TargetU;

use crate::args;
use crate::display;
use crate::errors::{AppError, AppResult, AppResultU};
use crate::types::*;



type Pid = Arc<Mutex<Option<u32>>>;


pub fn start() -> AppResultU {
    #[cfg(feature = "notification")]
    libnotify::init("axe").map_err(|_| AppError::Libnotify)?;

    let app_options = args::parse()?;

    let targets: Vec<TargetU> = app_options.targets.to_vec();
    let rx = patrol::spawn(targets);

    let pid = Arc::new(Mutex::<Option<u32>>::new(None));
    let mut changed: Option<PathBuf> = None;

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

        changed = Some(rx.recv().unwrap().path);
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

fn concrete(cl: &[Part], changed: Option<PathBuf>, targets: &[TargetU]) -> AppResult<Option<Vec<String>>> {
    cl.iter().map(|it| match it {
        Part::Literal(s) => Ok(Some(s.to_owned())),
        Part::Changed => changed.as_ref().map(path_to_string).transpose(),
        Part::Position(index) => if let Some(target) = targets.get(*index - 1) {
            Some(path_to_string(&target.path)).transpose()
        } else {
            Err(AppError::InvalidPosition(*index))
        }
    }).collect()
}

fn make_command(option: &AppOption, changed: Option<PathBuf>) -> AppResult<Option<(Command, String)>> {
    concrete(&option.command_line, changed, &option.targets)?.map(|command_line| {
        let (program, args) = command_line.split_first().ok_or(AppError::NotEnoughArguments)?;

        let mut command = Command::new(program);

        command.args(args);

        if let Some(stdin) = option.stdin.clone() {
            let stdin = File::open(stdin)?;
            command.stdin(stdin);
        }
        if let Some(stdout) = option.stdout.clone() {
            let mut oo = OpenOptions::new();
            oo.write(true).create(true);
            if option.append {
                oo.append(true);
            }
            let stdout = oo.open(stdout)?;
            command.stdout(stdout);
        }

        Ok((command, program.to_owned()))
    }).transpose()
}

fn path_to_string<T: AsRef<Path>>(path: &T) -> AppResult<String> {
    path.as_ref().to_str().ok_or(AppError::FilepathEncoding).map(|it| it.to_string())
}
