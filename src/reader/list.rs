use itertools::Itertools;

use crate::{
    form::{Form, FormKind},
    Error,
};
use std::iter::Peekable;

struct ListInner {
    start_symbol: &'static str,
    end_symbol: &'static str,
}

impl ListInner {
    pub fn new(start_symbol: &'static str, end_symbol: &'static str) -> ListInner {
        ListInner {
            start_symbol,
            end_symbol,
        }
    }

    pub fn read<'a, F>(
        &self,
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
        f: F,
    ) -> Option<Result<Form, Error>>
    where
        F: FnOnce(Vec<Form>) -> Result<Form, Error>,
    {
        assert_eq!(token_iter.next(), Some(self.start_symbol));
        let mut values = Vec::new();
        loop {
            if token_iter.peek() == Some(&self.end_symbol) {
                token_iter.next();
                break Some(f(values));
            }
            match super::read_form(token_iter) {
                Some(Ok(ast)) => values.push(ast),
                e @ Some(Err(_)) => break e,
                None => break Some(Err(Error::UnbalancedList)),
            }
        }
    }
}

pub fn read_list<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    ListInner::new("(", ")").read(token_iter, |values| Ok(Form::list(values)))
}

pub fn read_vector<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    ListInner::new("[", "]").read(token_iter, |values| Ok(Form::vector(values)))
}

pub fn read_hash_map<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Option<Result<Form, Error>> {
    ListInner::new("{", "}").read(token_iter, |values| {
        if values.len() % 2 == 1 {
            Err(Error::InvalidArgument)
        } else {
            Ok(Form {
                kind: FormKind::HashMap(values.into_iter().tuples().collect()),
                meta: None,
            })
        }
    })
}
