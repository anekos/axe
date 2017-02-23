
use ansi_term::Style;
use ansi_term::Colour::{White, Black, Red};
use chrono::{Local, DateTime};
use std::time::Duration;
use std::process::Command;


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

    println!("{}", Style::new().bold().on(White).fg(Black).paint(buf));
}


pub fn time(t: Duration) {
    let msec: u64 = t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1000000;
    let style = Style::new().bold().fg(Red);

    let s =
        if 60 * 1000 <= msec {
            format!("{} min {} sec", msec / 60 / 1000, msec % (60 * 1000) / 1000)
        } else {
            format!("{} sec", msec as f64 / 1000.0)
        };
    println!("{}", style.paint(s));
}


pub fn error(message: &str) {
    let style = Style::new().bold().on(Red);

    println!("{}", style.paint(message));
}
