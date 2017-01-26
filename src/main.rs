
extern crate inotify;
extern crate chrono;


mod notifier;


use chrono::{Local, DateTime};
use std::env::args;
use std::time::{Duration, Instant};
use std::sync::mpsc::channel;
use std::thread;
use std::process::Command;
use notifier::Target;


const SEP: &'static str = "--";


fn parse_arguments(a: Vec<String>) -> (Vec<Target>, Vec<String>) {
    fn sep(it: &String) -> bool { it == SEP }
    fn nsep(it: &String) -> bool { it != SEP }
    fn target(it: String) -> Target { Target::new(&it) }

    let b = a.clone();

    if a.iter().skip(1).any(sep) {
        (
            a.into_iter().take_while(nsep).map(target).collect(),
            b.into_iter().skip_while(nsep).skip(1).collect()
        )
    } else {
        (
            a.into_iter().take(1).map(target).collect(),
            b.into_iter().skip(1).collect()
        )
    }
}


fn puts_separator() {
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

    println!("[30;47;1m{}[0m", buf);
}


fn puts_time(t: Duration) {
    let msec: u64 = t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1000000;

    if 60 * 1000 <= msec {
        println!("{} min {} sec", msec / 60 / 1000, msec % (60 * 1000) / 1000)
    } else {
        println!("{} sec", msec as f64 / 1000.0);
    }
}


fn main() {
    let (targets, command) = parse_arguments(args().collect());

    // println!("targets: {:?}", targets);
    // println!("command: {:?}", command);

    if let Some(program) = command.first() {
        let args: Vec<&String> = command.iter().skip(1).collect();

        let (tx, rx) = channel();

        thread::spawn(move || {
            notifier::start_to_watch(targets, tx);
        });

        loop {
            let _ = rx.recv();
            puts_separator();

            let t = Instant::now();

            let mut child = Command::new(program).args(args.as_slice()).spawn().unwrap();
            child.wait().unwrap();

            puts_time(t.elapsed());

            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            }
        }
    };
}
