
use patrol::Target;



pub struct AppOption {
    pub sync: bool,
    pub command_line: Vec<Part>,
    pub signal: i32,
    pub targets: Vec<Target<String>>,
}

pub enum Part {
    Changed,
    Literal(String),
    Position(usize),
}
