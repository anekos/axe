
use std::env;
use std::fs;
use std::io::sink;
use std::path::PathBuf;

use patrol::Target;

use crate::errors::{AppError, AppResult};
use crate::types::*;



const SEP: &str = "--";
const PLACE_HOLDER: &str = "%";


pub fn parse() -> AppResult<AppOption> {
    let mut target_command: Vec<String> = vec![];
    let mut signal = libc::SIGTERM;
    let mut sync = false;

    {
        use argparse::{ArgumentParser, Collect, StoreConst, StoreTrue};
        let mut ap = ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.refer(&mut signal).add_option(&["--kill", "-k"], StoreConst(libc::SIGKILL), "Use KILL signal");
        ap.refer(&mut sync).add_option(&["--sync", "-s"], StoreTrue, "Do not use signal");
        ap.refer(&mut target_command).add_argument("Target/Command", Collect, "Target or command");
        let args = env::args().collect();
        ap.parse(args, &mut sink(), &mut sink()).map_err(|_| AppError::InvalidArgument)?;
    }

    let (targets, command_line) = split_params(target_command)?;
    let command_line: Vec<Part> = command_line.into_iter().map(|it| match it.as_ref() {
        PLACE_HOLDER => Part::Changed,
        _ => Part::Literal(it),
    }).collect();

    let targets = targets.into_iter().map(make_target).collect::<AppResult<Vec<Target<String>>>>()?;

    Ok(AppOption {
        command_line,
        signal,
        sync,
        targets,
    })
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

fn make_target(s: String) -> AppResult<Target<String>> {
    let path = PathBuf::from(&s);
    let path = fs::canonicalize(path)?;
    if !path.exists() {
        return Err(AppError::TargetNotFound(path.to_owned()));
    }
    Ok(Target::new(path, s))
}
