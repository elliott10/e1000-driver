use core::fmt;

use alloc::collections::TryReserveError;

#[derive(Debug)]
pub enum Error {
    Unknown(&'static str),
    Notsupport,
    Config(&'static str),
    Nvm(&'static str),
    TryReserveError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown(e) => write!(f, "error: {}", e),
            Error::Notsupport => write!(f, "error: not support"),
            Error::Config(e) => write!(f, "error: config error: {}", e),
            Error::Nvm(e) => write!(f, "nvm error: {}", e),
            Error::TryReserveError => write!(f, "try reserve error"),
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;

impl From<TryReserveError> for Error {
    fn from(_: TryReserveError) -> Self {
        Error::TryReserveError
    }
}
