use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take, take_while},
    character::complete::one_of,
    combinator::{iterator, recognize},
    error::ErrorKind,
    sequence::{delimited, preceded},
    IResult,
};

use crate::{RispError, RispForm};

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

fn parse_whitespace(input: &str) -> IResult<&str, &str> {
    take_while(is_whitespace)(input)
}

fn parse_special(input: &str) -> IResult<&str, &str> {
    alt((tag("~@"), recognize(one_of("[]{}()'`~^@"))))(input)
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    let string_escaped = escaped(is_not("\\\""), '\\', take(1usize));
    recognize(delimited(tag("\""), string_escaped, tag("\"")))(input)
}

fn parse_comment(input: &str) -> IResult<&str, &str> {
    recognize(preceded(tag(";"), is_not("\n\r")))(input)
}

fn parse_atom(input: &str) -> IResult<&str, &str> {
    is_not("[]{}()'\"`,; \t\n\r")(input)
}

pub fn tokenize(input: &str) -> IResult<&str, &str> {
    preceded(parse_whitespace, alt((
        parse_special,
        parse_string,
        parse_comment,
        parse_atom,
    )))(input)
}

fn is_list_start(s: &str) -> bool {
    one_of::<_, _, (&str, ErrorKind)>("({[")(s).is_ok()
}

fn is_list_end(s: &str) -> bool {
    one_of::<_, _, (&str, ErrorKind)>(")}]")(s).is_ok()
}

fn read_list(start_token: &str, end_token: &str, contents: Vec<RispForm>) -> Result<RispForm, RispError> {
    match start_token {
        "(" if end_token == ")" => Ok(RispForm::List(contents)),
        "{" if end_token == "}" => Ok(RispForm::List(contents)),
        "[" if end_token == "]" => Ok(RispForm::List(contents)),
        _ => Err(RispError::Reason("mismatched list delimiter".to_string())),
    }
}

fn read_atom(input: &str) -> Result<RispForm, RispError> {
    Ok(RispForm::Atom(input.to_string()))
}

pub fn read_str(input: &str) -> Result<RispForm, RispError> {
    let mut list_stack: Vec<(&str, Vec<RispForm>)> = vec![];
    for token in &mut iterator(input, tokenize) {
        assert!(!token.is_empty(), "'read_str' received empty token");
        if is_list_start(token) {
            list_stack.push((token, Vec::new()));
            continue;
        }

        let new_form = if is_list_end(token) {
            if let Some((start_token, contents)) = list_stack.pop() {
                read_list(start_token, token, contents)?
            } else {
                return Err(RispError::Reason("unbalanced list delimiter".into()));
            }
        } else {
            read_atom(token)?
        };

        if let Some((_, parent)) = list_stack.last_mut() {
            parent.push(new_form);
        } else {
            return Ok(new_form);
        }
    }
    Ok(RispForm::Nil)
}

#[derive(Debug)]
enum PrintStack<'a> {
    Form(&'a RispForm),
    Bracket(char),
}

pub fn pr_str(input: &RispForm) -> String {
    use PrintStack::*;
    // dbg!(input);
    let mut stack: Vec<PrintStack> = Vec::new();
    let mut output = String::new();
    stack.push(Form(input));
    while let Some(current) = stack.pop() {
        match current {
            Form(RispForm::Atom(content)) => output.push_str(content),
            Form(RispForm::Nil) => output.push_str("nil"),
            Form(RispForm::List(content)) => {
                output.push_str("(");
                stack.push(Bracket(')'));
                stack.extend(content.iter().rev().map(Form));
                continue;
            }
            Bracket(bracket) => output.push(bracket),
        }
        if matches!(stack.last(), Some(Form(_))) {
            output.push(' ');
        }
    }
    output
}
