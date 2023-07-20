// use std::{
//     collections::HashMap,
//     iter::Peekable,
//     str::{CharIndices, Chars},
// };

mod reader;

pub use reader::{read_str, pr_str};

#[derive(Clone, Debug)]
pub enum RispForm {
    Atom(String),
    List(Vec<RispForm>),
    Nil,
}

#[derive(Clone, Debug)]
pub enum RispError {
    Reason(String),
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
