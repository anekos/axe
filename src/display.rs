
use std::process::Command;
use std::time::Duration;

use chrono::{Local, DateTime};
use deco::*;


pub fn separator() {
    let now: DateTime<Local> = Local::now();

    let cols: usize =
        String::from_utf8(
            Command::new("tput").arg("cols").output().unwrap().stdout).unwrap()
        .trim()
        .parse().unwrap();

    let mut buf = format!("# {} ", now.format("%H:%M:%S"));

    for _ in 0..(cols - buf.len()) {
        buf.push('#');
    }

    deprintln!([bold on_white black "{}" !] buf);
}


pub fn time(t: Duration) {
    let msec: u64 = t.as_secs() * 1000 + u64::from(t.subsec_nanos()) / 1_000_000;

    let s =
        if 60 * 1000 <= msec {
            format!("{} min {} sec", msec / 60 / 1000, msec % (60 * 1000) / 1000)
        } else {
            format!("{} sec", msec as f64 / 1000.0)
        };
    deprintln!([bold red "{}"] s);
}


pub fn error(message: &str) {
    deprintln!([bold red "{}"] message);
}

pub fn killing(pid: u32) {
    deprintln!([bold red "Killing {} !"] pid);
}
