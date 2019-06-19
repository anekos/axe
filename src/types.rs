
use std::default::Default;
use std::path::PathBuf;

use patrol::Target;



pub struct AppOption {
    pub command_line: Vec<Part>,
    pub signal: i32,
    pub stdin: Option<PathBuf>,
    pub sync: bool,
    pub targets: Vec<Target<String>>,
}

pub enum Part {
    Changed,
    Literal(String),
    Position(usize),
}



impl Default for AppOption {
    fn default() -> Self {
        Self {
            command_line: vec![],
            signal: libc::SIGTERM,
            stdin: None,
            sync: false,
            targets: vec![],
        }
    }
}
