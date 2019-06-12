
use patrol::Target;



pub struct AppOption {
    pub command_line: Vec<String>,
    pub signal: i32,
    pub targets: Vec<Target>,
}
