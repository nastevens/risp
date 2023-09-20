use std::{cell::RefCell, rc::Rc};

use crate::{Env, Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn from_str(name: &str) -> Ident {
        Ident { name: name.into() }
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Atom {
    pub value: Rc<RefCell<Form>>,
}

impl Atom {
    pub fn new(form: Form) -> Atom {
        Atom {
            value: Rc::new(RefCell::new(form)),
        }
    }
}

impl From<Atom> for Form {
    fn from(atom: Atom) -> Form {
        Form {
            kind: FormKind::Atom(atom),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Form {
    pub kind: FormKind,
}

macro_rules! form_predicate_fn {
    ($method:ident, $kind:pat) => {
        pub fn $method(&self) -> bool {
            matches!(self, Form { kind: $kind })
        }
    };
}

impl Form {
    form_predicate_fn!(is_nil, FormKind::Nil);
    form_predicate_fn!(is_boolean, FormKind::Boolean(_));
    form_predicate_fn!(is_symbol, FormKind::Symbol(_));
    form_predicate_fn!(is_number, FormKind::Integer(_) | FormKind::Float(_));
    form_predicate_fn!(is_string, FormKind::String(_));
    form_predicate_fn!(is_keyword, FormKind::Keyword(_));
    form_predicate_fn!(is_list, FormKind::List(_));
    form_predicate_fn!(is_vector, FormKind::Vector(_));
    form_predicate_fn!(is_hash_map, FormKind::HashMap(_));
    form_predicate_fn!(is_native_fn, FormKind::NativeFn(_));
    form_predicate_fn!(is_user_fn, FormKind::UserFn { .. });
    form_predicate_fn!(is_atom, FormKind::Atom(_));

    pub fn nil() -> Form {
        Form {
            kind: FormKind::Nil,
        }
    }

    pub fn boolean(value: bool) -> Form {
        Form {
            kind: FormKind::Boolean(value),
        }
    }

    pub fn symbol(name: &str) -> Form {
        Form {
            kind: FormKind::Symbol(Ident::from_str(name)),
        }
    }

    pub fn int(value: i64) -> Form {
        Form {
            kind: FormKind::Integer(value),
        }
    }

    pub fn float(value: f64) -> Form {
        Form {
            kind: FormKind::Float(value),
        }
    }

    pub fn string(value: impl Into<String>) -> Form {
        Form {
            kind: FormKind::String(value.into()),
        }
    }

    pub fn keyword(value: impl Into<String>) -> Form {
        Form {
            kind: FormKind::Keyword(value.into()),
        }
    }

    pub fn list(value: impl IntoIterator<Item = Form>) -> Form {
        Form {
            kind: FormKind::List(value.into_iter().collect()),
        }
    }

    pub fn is_empty_list(&self) -> bool {
        matches!(self.kind, FormKind::List(ref inner) if inner.is_empty())
    }

    pub fn vector(value: impl IntoIterator<Item = Form>) -> Form {
        Form {
            kind: FormKind::Vector(value.into_iter().collect()),
        }
    }

    pub fn hash_map(value: impl IntoIterator<Item = Form>) -> Form {
        Form {
            kind: FormKind::HashMap(value.into_iter().collect()),
        }
    }

    pub fn native_fn(f: &'static dyn Fn(Form) -> Result<Form>) -> Form {
        Form {
            kind: FormKind::NativeFn(f),
        }
    }

    pub fn user_fn(binds: Vec<Ident>, bind_rest: Option<Ident>, body: Form, env: Env) -> Form {
        Form {
            kind: FormKind::UserFn {
                binds,
                bind_rest,
                body: Box::new(body),
                env,
                is_macro: false,
            },
        }
    }

    pub fn atom(atom: Atom) -> Form {
        Form {
            kind: FormKind::Atom(atom),
        }
    }

    pub fn macro_(binds: Vec<Ident>, bind_rest: Option<Ident>, body: Form, env: Env) -> Form {
        Form {
            kind: FormKind::UserFn {
                binds,
                bind_rest,
                body: Box::new(body),
                env,
                is_macro: true,
            },
        }
    }

    pub fn iter(&self) -> Result<impl Iterator<Item = &Form>> {
        match &self.kind {
            FormKind::List(inner) => Ok(inner.iter()),
            FormKind::Vector(inner) => Ok(inner.iter()),
            _ => Err(Error::NotIterable),
        }
    }

    pub fn iter_mut(&mut self) -> Result<impl Iterator<Item = &mut Form>> {
        match &mut self.kind {
            FormKind::List(inner) => Ok(inner.iter_mut()),
            FormKind::Vector(inner) => Ok(inner.iter_mut()),
            _ => Err(Error::NotIterable),
        }
    }

    pub fn try_into_iter(self) -> Result<impl Iterator<Item = Form> + DoubleEndedIterator> {
        match self.kind {
            FormKind::List(inner) => Ok(inner.into_iter()),
            FormKind::Vector(inner) => Ok(inner.into_iter()),
            _ => Err(Error::NotIterable),
        }
    }

    pub fn is_symbol_named(&self, test: &str) -> bool {
        matches!(&self.kind, FormKind::Symbol(Ident { name }) if name == test)
    }

    pub fn is_sequential(&self) -> bool {
        matches!(&self.kind, FormKind::List(_) | FormKind::Vector(_))
    }

    pub fn is_empty_sequential(&self) -> bool {
        matches!(&self.kind, FormKind::List(inner) | FormKind::Vector(inner) if inner.is_empty())
    }

    pub fn empty_list() -> Form {
        Form {
            kind: FormKind::List(vec![]),
        }
    }
}

#[derive(Clone)]
pub enum FormKind {
    Nil,
    Boolean(bool),
    Symbol(Ident),
    Integer(i64),
    Float(f64),
    String(String),
    Keyword(String),
    List(Vec<Form>),
    Vector(Vec<Form>),
    HashMap(Vec<Form>),
    NativeFn(&'static dyn Fn(Form) -> Result<Form>),
    UserFn {
        binds: Vec<Ident>,
        bind_rest: Option<Ident>,
        body: Box<Form>,
        env: Env,
        is_macro: bool,
    },
    Atom(Atom),
}

impl PartialEq<FormKind> for FormKind {
    fn eq(&self, other: &FormKind) -> bool {
        match (self, other) {
            (FormKind::Nil, FormKind::Nil) => true,
            (FormKind::Boolean(a), FormKind::Boolean(b)) => *a == *b,
            (FormKind::Symbol(a), FormKind::Symbol(b)) => *a == *b,
            (FormKind::Integer(a), FormKind::Integer(b)) => *a == *b,
            (FormKind::Float(a), FormKind::Float(b)) => *a == *b,
            (FormKind::String(a), FormKind::String(b)) => *a == *b,
            (FormKind::Keyword(a), FormKind::Keyword(b)) => *a == *b,
            (FormKind::List(a) | FormKind::Vector(a), FormKind::List(b) | FormKind::Vector(b)) => {
                *a == *b
            }
            (FormKind::HashMap(a), FormKind::HashMap(b)) => *a == *b,
            (FormKind::NativeFn(_), _) => false,
            (FormKind::UserFn { .. }, _) => false,
            (_, _) => false,
        }
    }
}
