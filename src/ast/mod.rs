mod list;

use crate::RispError;

pub use self::list::{HashMap, List, Vector};

pub trait Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result;

    fn get(&self, _index: usize) -> Option<&Ast> {
        None
    }
}

pub struct Ast {
    inner: Box<dyn Form>,
}

impl Ast {
    pub fn of(input: impl Form + 'static) -> Ast {
        Self {
            inner: Box::new(input),
        }
    }
}

impl Form for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result {
        Form::fmt(&*self.inner, f, print_readably)
    }

    fn get(&self, index: usize) -> Option<&Ast> {
        Form::get(&*self.inner, index)
    }
}

#[derive(Debug)]
pub struct Nil;

impl Form for Nil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _print_readably: bool) -> std::fmt::Result {
        f.write_str("nil")
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub(crate) value: String,
}

impl Symbol {
    pub fn with_value(value: impl Into<String>) -> Ast {
        Ast::of(Symbol {
            value: value.into(),
        })
    }
}

impl Form for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _print_readably: bool) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

#[derive(Debug)]
pub struct RString {
    pub(crate) value: String,
}

impl RString {
    pub fn with_value(value: impl Into<String>) -> Ast {
        Ast::of(RString {
            value: value.into(),
        })
    }

    pub fn expand(&self) -> Result<String, RispError> {
        use nom::{
            branch::alt,
            bytes::complete::{escaped_transform, is_not, tag},
            combinator::value,
            sequence::delimited,
            IResult,
        };
        fn extract_and_expand(input: &str) -> IResult<&str, String> {
            alt((
                value(String::new(), tag("\"\"")),
                delimited(
                    tag("\""),
                    escaped_transform(
                        is_not("\\\""),
                        '\\',
                        alt((
                            value("\\", tag("\\")),
                            value("\"", tag("\"")),
                            value("\n", tag("n")),
                        )),
                    ),
                    tag("\""),
                ),
            ))(input)
        }
        Ok(extract_and_expand(&self.value)
            .map(|result| result.1)
            .map_err(|_| RispError::Eof)?)
    }
}

impl Form for RString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result {
        if print_readably {
            f.write_str(&self.value)
        } else {
            f.write_str(&self.expand().unwrap())
        }
    }
}

#[derive(Debug)]
pub struct Keyword {
    pub(crate) value: String,
}

impl Keyword {
    pub fn with_value(value: impl Into<String>) -> Ast {
        Ast::of(Keyword {
            value: value.into(),
        })
    }
}

impl Form for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _print_readably: bool) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}
