use std::io;

use neon::result::Throw;

pub type ViiruResult = Result<(), ViiruError>;

#[derive(Debug)]
pub enum ViiruError {
    JsThrow(Throw),
    IoError(io::Error),
}

impl From<Throw> for ViiruError {
    fn from(throw: Throw) -> Self {
        ViiruError::JsThrow(throw)
    }
}

impl From<io::Error> for ViiruError {
    fn from(err: io::Error) -> Self {
        ViiruError::IoError(err)
    }
}
