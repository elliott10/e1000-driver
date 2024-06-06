use core::fmt;

#[derive(Debug)]
pub enum Error {
    Unknown(&'static str),
    Notsupport,
    Config(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown(e) => write!(f, "error: {}", e),
            Error::Notsupport => write!(f, "error: not support"),
            Error::Config(e) => write!(f, "error: config error: {}", e),
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;
