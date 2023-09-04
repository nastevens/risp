mod convert;
pub mod core;
mod env;
pub mod eval;
pub mod exec;
pub mod form;
pub mod format;
// mod ptr;
mod reader;

use std::{convert::Infallible, num::TryFromIntError};

pub use env::Env;
pub use form::{Form, FormKind};
pub use eval::eval;
pub use format::pr_str;
pub use reader::read_str;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("unexpected end of input")]
    Eof,
    #[error("unbalanced list")]
    UnbalancedList,
    #[error("{0} not found")]
    UnknownSymbol(String),
    #[error("invalid number {0}")]
    InvalidNumber(String),
    #[error("invalid argument")]
    InvalidArgument,
    #[error("serde error {0}")]
    SerdeError(String),
    #[error("tried to apply something that's not a function")]
    InvalidApply,
    #[error("could not convert integer")]
    NumberConversion,
    #[error("attempted to iterate non-iterable Form")]
    NotIterable,
}

impl From<Infallible> for Error {
    fn from(x: Infallible) -> Error {
        match x {}
    }
}

impl From<TryFromIntError> for Error {
    fn from(_value: TryFromIntError) -> Self {
        Error::NumberConversion
    }
}

fn eval_str(s: &str, env: &mut Env) {
    let parsed = read_str(s).unwrap_or_else(|e| panic!("Could not parse eval_str: {e}"));
    eval(parsed, env).unwrap_or_else(|e| panic!("eval_str input '{s}' failed: {e}"));
}

