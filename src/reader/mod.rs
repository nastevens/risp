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
    form::{Form, FormKind, Ident},
    Error,
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

fn read_nil<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    token_iter.next().map(|value| {
        assert_eq!(value, "nil");
        Ok(Form::nil())
    })
}

fn read_bool<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    token_iter.next().map(|value| match value {
        "true" => Ok(Form::boolean(true)),
        "false" => Ok(Form::boolean(false)),
        s => panic!("not a boolean: {}", s),
    })
}

fn read_symbol<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    token_iter.next().map(|name| Ok(Form::symbol(name)))
}

fn read_number<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    token_iter.next().map(|s| {
        str::parse::<i64>(s)
            .map(|n| Form {
                kind: FormKind::Integer(n),
                meta: None,
            })
            .map_err(|_| Error::InvalidNumber(s.into()))
    })
}

fn read_string<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    use aho_corasick::AhoCorasick;
    use std::sync::OnceLock;
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    const PATTERNS: &[&str] = &["\\\\", "\\n", "\\\""];
    const REPLACEMENTS: &[&str] = &["\\", "\n", "\""];
    token_iter.next().map(|value| {
        let no_quotes = &value[1..(value.len() - 1)];
        let escape_replacment = AC.get_or_init(|| {
            AhoCorasick::new(PATTERNS).expect("parsing static AhoCorasick patterns")
        });
        let escaped = escape_replacment.replace_all(no_quotes, REPLACEMENTS);
        Ok(Form {
            kind: FormKind::String(escaped),
            meta: None,
        })
    })
}

fn read_keyword<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    token_iter.next().map(|name| {
        let no_colon = name.chars().skip(1).collect::<String>();
        Ok(Form {
            kind: FormKind::Keyword(no_colon),
            meta: None,
        })
    })
}

fn reader_macro<'a>(
    fnname: &str,
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Form, Error> {
    token_iter.next();
    let mut values = vec![Form {
        kind: FormKind::Symbol(Ident::from_str(fnname)),
        meta: None,
    }];
    match read_form(token_iter) {
        Some(Ok(form_result)) => values.push(form_result),
        Some(err @ Err(_)) => return err,
        None => return Err(Error::Eof),
    }
    Ok(Form {
        kind: FormKind::List(values),
        meta: None,
    })
}

fn meta_reader_macro<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<Form, Error> {
    assert_eq!(token_iter.next(), Some("^"));
    let meta = read_form(token_iter).transpose()?.ok_or(Error::Eof)?;
    let form = read_form(token_iter).transpose()?.ok_or(Error::Eof)?;
    let symbol = Form::symbol("with-meta");
    Ok(Form::list([symbol, form, meta]))
}

fn read_form<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    match token_iter.peek() {
        Some(&"nil") => read_nil(token_iter),
        Some(&"true") | Some(&"false") => read_bool(token_iter),
        Some(&"(") => self::list::read_list(token_iter),
        Some(&"[") => self::list::read_vector(token_iter),
        Some(&"{") => self::list::read_hash_map(token_iter),
        Some(&"'") => Some(reader_macro("quote", token_iter)),
        Some(&"`") => Some(reader_macro("quasiquote", token_iter)),
        Some(&"~") => Some(reader_macro("unquote", token_iter)),
        Some(&"~@") => Some(reader_macro("splice-unquote", token_iter)),
        Some(&"@") => Some(reader_macro("deref", token_iter)),
        Some(&"^") => Some(meta_reader_macro(token_iter)),
        Some(s) if str::parse::<i64>(s).is_ok() => read_number(token_iter),
        Some(s) if s.starts_with('"') => read_string(token_iter),
        Some(s) if s.starts_with(':') => read_keyword(token_iter),
        Some(_token) => read_symbol(token_iter),
        None => None,
    }
}

pub fn read_str(input: &str) -> Result<Form, Error> {
    let mut parser = iterator(input, tokenize);
    let ast = {
        let mut fused = (&mut parser).fuse();
        let ast = {
            let mut iter = Iterator::peekable(&mut fused);
            read_form(&mut iter).ok_or(Error::Eof)
        }?;
        match fused.next() {
            Some(_) => Err(Error::Eof),
            None => Ok(ast),
        }
    }?;
    match parser.finish() {
        Ok(("", ())) => ast,
        _ => Err(Error::Eof),
    }
}
