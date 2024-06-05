use core::fmt;

#[derive(Debug)]
pub enum Error {
    Unknown(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown(e) => write!(f, "error: {}", e),
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;
