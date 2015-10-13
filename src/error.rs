/// This is the standard error type used throughout this crate
/// Implementation based on suggestions in http://blog.burntsushi.net/rust-error-handling

use std::error::Error;
use std::fmt;
use std::io;

// TODO: Probably rename this
#[derive(Debug, Clone)]
pub enum StdError {
    Io(String),
    Eof
}

impl Error for StdError {
    fn description(&self) -> &str {
        match *self {
            StdError::Io(ref string) => string,
            StdError::Eof => "end of file"
        }
    }
}

impl fmt::Display for StdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StdError::Io(ref string) => write!(f, "IO error: {}", string),
            StdError::Eof => write!(f, "Error: End Of File")
        }
    }
}

/// Need to implement from so we can use try!
impl From<io::Error> for StdError {
    fn from(err: io::Error) -> StdError {
        let s = format!("{}", err);
        StdError::Io(s)
    }
}
