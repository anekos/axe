
use failure::Fail;



pub type AppResult<T> = Result<T, AppError>;
// pub type AppResultU = Result<(), AppError>;



#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "IO Error: {}", 0)]
    Io(std::io::Error),
    #[fail(display = "Not enough arguments")]
    NotEnoughArguments,
    #[fail(display = "Invalid number: {}", 0)]
    NumberFormat(std::num::ParseIntError),
    #[fail(display = "Target not found: {}", 0)]
    TargetNotFound(String),
}


macro_rules! define_error {
    ($source:ty, $kind:ident) => {
        impl From<$source> for AppError {
            fn from(error: $source) -> AppError {
                AppError::$kind(error)
            }
        }
    }
}

define_error!(std::io::Error, Io);
define_error!(std::num::ParseIntError, NumberFormat);