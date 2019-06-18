
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
