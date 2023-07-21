// use std::{
//     collections::HashMap,
//     iter::Peekable,
//     str::{CharIndices, Chars},
// };

pub mod ast;
mod reader;

use thiserror::Error;
pub use reader::{pr_str, read_str};

#[derive(Clone, Debug)]
pub enum ListKind {
    List,
    Vector,
    HashMap,
}

#[derive(Clone, Debug)]
pub enum RawForm {
    Atom(String),
    List(ListKind, Vec<RawForm>),
}

#[derive(Clone, Debug, Error)]
pub enum RispError {
    #[error("{0}")]
    Reason(String),
    #[error("unexpected end of input")]
    Eof,
    #[error("unbalanced list")]
    UnclosedList,
}

// #[derive(Clone, Debug)]
// pub struct RispEnv {
//     data: HashMap<RispIdent, RispToken>,
// }

#[cfg(test)]
mod test {
    // const STEP1_CASES: &[(&str, &str)] = &[("1", "1"), ("7", "7")];

    // #[test]
    // fn test_tokenize() {
    //     for (input, output) in STEP1_CASES {
    //         assert_eq!(output, super::tokenize(input));
    //     }
    // }
}
