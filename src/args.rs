
use std::env;
use std::fs;
use std::io::sink;
use std::path::PathBuf;

use patrol::{Target, TargetU};

use crate::errors::{AppError, AppResult};
use crate::types::*;



const SEP: &str = "--";


pub fn parse() -> AppResult<AppOption> {
    let mut option = AppOption::default();
    let mut target_command: Vec<String> = vec![];

    {
        use argparse::{ArgumentParser, Collect, StoreConst, StoreTrue, StoreOption};
        let mut ap = ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.refer(&mut option.signal).add_option(&["--kill", "-k"], StoreConst(libc::SIGKILL), "Use KILL signal");
        ap.refer(&mut option.sync).add_option(&["--sync", "-s"], StoreTrue, "Do not use signal");
        ap.refer(&mut option.stdin).add_option(&["--stdin", "-i"], StoreOption, "File path for stdin");
        ap.refer(&mut option.stdout).add_option(&["--stdout", "-o"], StoreOption, "File path for stdout");
        ap.refer(&mut option.append).add_option(&["--append", "-a"], StoreTrue, "`--stdout` with append mode");
        ap.refer(&mut option.delay).add_option(&["--delay", "-d"], StoreOption, "Delay time (msec) before run");
        ap.refer(&mut option.env).add_option(&["--env", "-e"], Collect, "Delay time (msec) before run");
        ap.refer(&mut target_command).add_argument("Target/Command", Collect, "Target or command");
        let args = env::args().collect();
        ap.parse(args, &mut sink(), &mut sink()).map_err(|_| AppError::InvalidArgument)?;
    }

    let (targets, command_line) = split_params(target_command)?;
    option.command_line = command_line.into_iter().map(|it| {
        if it == "%%" {
            return Part::Literal("%".to_owned())
        }
        if let Some(stripped) = it.strip_prefix('%') {
            if stripped.is_empty() {
                return Part::Changed;
            }
            if let Ok(index) = stripped.parse() {
                if 0 < index {
                    return Part::Position(index)
                }
            }
        }
        Part::Literal(it)
    }).collect();

    option.targets = targets.into_iter().map(make_target).collect::<AppResult<Vec<TargetU>>>()?;

    Ok(option)
}

fn split_params(a: Vec<String>) -> AppResult<(Vec<String>, Vec<String>)> {
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

fn make_target(s: String) -> AppResult<TargetU> {
    let path = PathBuf::from(&s);
    let path = fs::canonicalize(path)?;
    if !path.exists() {
        return Err(AppError::TargetNotFound(path));
    }
    Ok(Target::new(path, ()))
}
