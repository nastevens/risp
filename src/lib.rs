// use std::{rc::Rc, any::Any};

pub mod ast;
pub mod core;
pub mod eval;
pub mod exec;
pub mod format;
mod de;
mod env;
// mod ptr;
mod reader;

pub use env::Env;
pub use format::pr_str;
pub use reader::read_str;
pub use ast::{Form, FormKind};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, crate::Error>;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("unexpected end of input")]
    Eof,
    #[error("unbalanced list")]
    UnbalancedList,
    #[error("unknown symbol {0}")]
    UnknownSymbol(String),
    #[error("invalid number {0}")]
    InvalidNumber(String),
    #[error("invalid argument")]
    InvalidArgument,
    #[error("serde error {0}")]
    SerdeError(String),
    #[error("tried to apply something that's not a function")]
    InvalidApply,
}
