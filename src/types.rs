
use patrol::Target;



pub struct AppOption {
    pub sync: bool,
    pub command_line: Vec<String>,
    pub signal: i32,
    pub targets: Vec<Target>,
}
