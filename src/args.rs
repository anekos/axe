
use std::env;
use std::fs;
use std::io::sink;
use std::path::{Path, PathBuf};

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

    fn target(it: &str) -> Target { Target::new(it) }

    let (targets, mut command_line) = extract_params(target_command)?;

    if let Some(first) = targets.first() {
        for it in command_line.as_mut_slice().iter_mut() {
            if *it == PLACE_HOLDER {
                *it = first.clone()
            }
        }
    }

    let targets: Vec<String> = targets.iter().map(String::as_ref).map(to_absolute_path).collect();
    for it in &targets {
        if !Path::new(it).exists() {
            return Err(AppError::TargetNotFound(it.to_owned()));
        }
    }

    Ok(AppOption {
        command_line,
        signal,
        sync,
        targets: targets.iter().map(String::as_ref).map(target).collect(),
    })
}

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

fn to_absolute_path(path: &str) -> String {
    let buf = PathBuf::from(path);
    fs::canonicalize(buf).map(|it| it.to_str().unwrap().to_string()).unwrap_or_else(|_| path.to_owned())
}
