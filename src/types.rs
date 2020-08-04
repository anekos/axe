
use std::default::Default;
use std::path::PathBuf;

use patrol::TargetU;



pub struct AppOption {
    pub append: bool,
    pub command_line: Vec<Part>,
    pub delay: Option<u64>,
    pub env: Vec<String>,
    pub signal: i32,
    pub stdin: Option<PathBuf>,
    pub stdout: Option<PathBuf>,
    pub sync: bool,
    pub targets: Vec<TargetU>,
}

pub enum Part {
    Changed,
    Literal(String),
    Position(usize),
}



impl Default for AppOption {
    fn default() -> Self {
        Self {
            append: false,
            command_line: vec![],
            delay: None,
            env: vec![],
            signal: libc::SIGTERM,
            stdin: None,
            stdout: None,
            sync: false,
            targets: vec![],
        }
    }
}
