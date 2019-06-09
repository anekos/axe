
use std::path::{Path};

use patrol::Target;

use crate::errors::{AppError, AppResult};
use crate::types::*;



const SEP: &str = "--";
const PLACE_HOLDER: &str = "%";


pub fn parse(a: Vec<String>) -> AppResult<Parsed> {
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
