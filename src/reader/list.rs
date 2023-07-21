use std::iter::Peekable;

use crate::{ast::{Ast, self}, RispError};

use super::ReadForm;

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
    ) -> Option<Result<Ast, RispError>>
    where
        F: FnOnce(Vec<Ast>) -> Ast,
    {
        assert_eq!(token_iter.next(), Some(self.start_symbol));
        let mut values = Vec::new();
        loop {
            if token_iter.peek() == Some(&self.end_symbol) {
                token_iter.next();
                break Some(Ok(f(values)));
            }
            match super::read_form(token_iter) {
                Some(Ok(ast)) => values.push(ast),
                e @ Some(Err(_)) => break e,
                None => break Some(Err(RispError::UnclosedList)),
            }
        }
    }
}

impl ReadForm for ast::List {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        ListInner::new("(", ")").read(token_iter, ast::List::with_values)
    }

}

impl ReadForm for ast::Vector {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        ListInner::new("[", "]").read(token_iter, ast::Vector::with_values)
    }
}

impl ReadForm for ast::HashMap {
    fn read_form<'a>(
        token_iter: &mut Peekable<impl Iterator<Item = &'a str>>,
    ) -> Option<Result<Ast, RispError>>
    where
        Self: Sized,
    {
        ListInner::new("{", "}").read(token_iter, ast::HashMap::with_values)
    }
}
