use std::iter::Peekable;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_while1},
    character::complete::one_of,
    combinator::{iterator, recognize},
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult,
};

use crate::{
    ast::{self, Ast},
    RispError,
};

mod list;

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

fn parse_whitespace(input: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(input)
}

fn parse_comment(input: &str) -> IResult<&str, &str> {
    recognize(preceded(tag(";"), is_not("\n\r")))(input)
}

fn parse_ignored(input: &str) -> IResult<&str, ()> {
    fold_many0(alt((parse_whitespace, parse_comment)), || (), |_, _| ())(input)
}

fn parse_special(input: &str) -> IResult<&str, &str> {
    alt((tag("~@"), recognize(one_of("[]{}()'`~^@"))))(input)
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    alt((
        tag("\"\""),
        recognize(delimited(
            tag("\""),
            escaped(is_not("\\\""), '\\', one_of("n\\\"")),
            tag("\""),
        )),
    ))(input)
}

fn parse_atom(input: &str) -> IResult<&str, &str> {
    is_not("[]{}()'\"`,; \t\n\r")(input)
}

pub fn tokenize(input: &str) -> IResult<&str, &str> {
    delimited(
        parse_ignored,
        alt((parse_special, parse_string, parse_atom)),
        parse_ignored,
    )(input)
}

fn reader_macro<'a>(
    fnname: &str,
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Ast, RispError> {
    token_iter.next();
    let mut values = vec![ast::Symbol::with_value(fnname)];
    match read_form(token_iter) {
        Some(Ok(form_result)) => values.push(form_result),
        Some(err @ Err(_)) => return err,
        None => return Err(RispError::Eof),
    }
    Ok(ast::List::with_values(values))
}

fn meta_reader_macro<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Ast, RispError> {
    assert_eq!(token_iter.next(), Some("^"));
    let meta = read_form(token_iter).transpose()?.ok_or(RispError::Eof)?;
    let form = read_form(token_iter).transpose()?.ok_or(RispError::Eof)?;
    Ok(ast::List::with_values([ast::Symbol::with_value("with-meta"), form, meta]))
}

fn read_form<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Ast, RispError>> {
    match token_iter.peek() {
        Some(&"nil") => <ast::Nil as ReadForm>::read_form(token_iter),
        Some(&"(") => <ast::List as ReadForm>::read_form(token_iter),
        Some(&"[") => <ast::Vector as ReadForm>::read_form(token_iter),
        Some(&"{") => <ast::HashMap as ReadForm>::read_form(token_iter),
        Some(&"'") => Some(reader_macro("quote", token_iter)),
        Some(&"`") => Some(reader_macro("quasiquote", token_iter)),
        Some(&"~") => Some(reader_macro("unquote", token_iter)),
        Some(&"~@") => Some(reader_macro("splice-unquote", token_iter)),
        Some(&"@") => Some(reader_macro("deref", token_iter)),
        Some(&"^") => Some(meta_reader_macro(token_iter)),
        Some(s) if s.starts_with('"') => <ast::RString as ReadForm>::read_form(token_iter),
        Some(s) if s.starts_with(':') => <ast::Keyword as ReadForm>::read_form(token_iter),
        Some(_token) => <ast::Symbol as ReadForm>::read_form(token_iter),
        None => None,
    }
}

pub trait ReadForm {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized;
}

impl ReadForm for ast::Nil {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        assert_eq!(token_iter.next(), Some("nil"));
        Some(Ok(Ast::of(ast::Nil)))
    }
}

impl ReadForm for ast::Symbol {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        token_iter.next().map(ast::Symbol::with_value).map(Ok)
    }
}


// fn extract_meat(input: &str) -> IResult<&str, String> {
//     if input == "" {
//         return Ok(("", String::new()));
//     }
//     alt((
//         value(String::new(), tag("\"\"")),
//         delimited(
//             tag("\""),
//             escaped_transform(
//                 is_not("\\\""),
//                 '\\',
//                 alt((
//                     value("\\", tag("\\")),
//                     value("\"", tag("\"")),
//                     value("\n", tag("n")),
//                 )),
//             ),
//             tag("\""),
//         ),
//     ))(input)
// }

impl ReadForm for ast::RString {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        token_iter.next().map(ast::RString::with_value).map(Ok)
    }
}

impl ReadForm for ast::Keyword {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        token_iter.next().map(ast::Keyword::with_value).map(Ok)
    }
}

pub fn read_str(input: &str) -> Result<Ast, RispError> {
    let mut parser = iterator(input, tokenize);
    let ast = {
        let mut fused = (&mut parser).fuse();
        let ast = {
            let mut iter = Iterator::peekable(&mut fused);
            read_form(&mut iter).ok_or(RispError::Eof)
        }?;
        match fused.next() {
            Some(_) => Err(RispError::Eof),
            None => Ok(ast),
        }
    }?;
    match parser.finish() {
        Ok(("", ())) => ast,
        _ => Err(RispError::Eof),
    }
}

struct PrintFormat<'a>(&'a Ast);

impl<'a> std::fmt::Display for PrintFormat<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ast::Form::fmt(self.0, f, true)
    }
}

pub fn pr_str(input: &Ast) -> String {
    format!("{}", PrintFormat(input))
}

