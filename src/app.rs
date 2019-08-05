use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

use libc;
#[cfg(feature = "notification")]
use libnotify::{Notification, Urgency};
use patrol::{Config, Patrol, TargetU};

use crate::display;
use crate::errors::{AppError, AppResult, AppResultU};
use crate::process::Process;
use crate::types::*;



pub fn start(app_options: AppOption, process: Process) -> AppResultU {
    let targets: Vec<TargetU> = app_options.targets.to_vec();
    let patrol = Patrol::new(Config { watch_new_directory: true }, targets);
    let rx = patrol.spawn();

    let mut changed: Option<PathBuf> = None;

    loop {
        process.terminate()?;

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
                let process = process.clone();
                thread::spawn(move || {
                    match command.spawn() {
                        Ok(mut child) => {
                            process.set(child.id());
                            on_exit(child.wait(), t, &program, Some(&process));
                            process.release();
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

fn on_exit(status: io::Result<ExitStatus>, at_start: Instant, program: &str, process: Option<&Process>) {
    match status {
        Ok(status) => match status.code() {
            Some(0) | None => {
                if let Some(process) = process {
                    if process.is_empty() {
                        return;
                    }
                }
                notify(&format!("OK - {}", program));
            },
            Some(code) => {
                display::status_code(code);
                notify(&format!("[{}] - {}", code, program));
            }
        },
        Err(ref err) if err.raw_os_error() == Some(libc::ECHILD) =>
            (),
        Err(err) =>
            display::error(&format!("Failed: {} {:?} {:?}", err, err.kind(), err.raw_os_error())),

    }
    display::time(at_start.elapsed());
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
        let program = command_line.first().ok_or(AppError::NotEnoughArguments)?;

        let mut command = Command::new("setsid");

        command.args(&command_line);

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
