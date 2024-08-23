use std::io;

use neon::{prelude::*, result::Throw};

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

pub fn return_or_throw<'a>(
    cx: &mut FunctionContext<'a>,
    result: ViiruResult,
) -> JsResult<'a, JsUndefined> {
    match result {
        Ok(()) => Ok(cx.undefined()),
        Err(ViiruError::JsThrow(throw)) => Err(throw),
        Err(ViiruError::IoError(err)) => cx.throw_error(err.to_string()),
    }
}
