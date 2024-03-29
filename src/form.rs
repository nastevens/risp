use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

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
        Form::atom(atom)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Form {
    pub kind: FormKind,
    pub meta: Option<Box<Form>>,
}

macro_rules! form_predicate_fn {
    ($method:ident, $kind:pat) => {
        pub fn $method(&self) -> bool {
            matches!(self, Form { kind: $kind, .. })
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
    form_predicate_fn!(is_atom, FormKind::Atom(_));

    pub fn nil() -> Form {
        Form {
            kind: FormKind::Nil,
            meta: None,
        }
    }

    pub fn boolean(value: bool) -> Form {
        Form {
            kind: FormKind::Boolean(value),
            meta: None,
        }
    }

    pub fn symbol(name: &str) -> Form {
        Form {
            kind: FormKind::Symbol(Ident::from_str(name)),
            meta: None,
        }
    }

    pub fn int(value: i64) -> Form {
        Form {
            kind: FormKind::Integer(value),
            meta: None,
        }
    }

    pub fn float(value: f64) -> Form {
        Form {
            kind: FormKind::Float(value),
            meta: None,
        }
    }

    pub fn string(value: impl Into<String>) -> Form {
        Form {
            kind: FormKind::String(value.into()),
            meta: None,
        }
    }

    pub fn keyword(value: impl Into<String>) -> Form {
        Form {
            kind: FormKind::Keyword(value.into()),
            meta: None,
        }
    }

    pub fn list(value: impl IntoIterator<Item = Form>) -> Form {
        Form {
            kind: FormKind::List(value.into_iter().collect()),
            meta: None,
        }
    }

    pub fn is_empty_list(&self) -> bool {
        matches!(self.kind, FormKind::List(ref inner) if inner.is_empty())
    }

    pub fn vector(value: impl IntoIterator<Item = Form>) -> Form {
        Form {
            kind: FormKind::Vector(value.into_iter().collect()),
            meta: None,
        }
    }

    pub fn hash_map(value: HashMap<Form, Form>) -> Form {
        Form {
            kind: FormKind::HashMap(value),
            meta: None,
        }
    }

    pub fn native_fn(f: &'static dyn Fn(Form) -> Result<Form>) -> Form {
        Form {
            kind: FormKind::NativeFn(f),
            meta: None,
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
            meta: None,
        }
    }

    pub fn atom(atom: Atom) -> Form {
        Form {
            kind: FormKind::Atom(atom),
            meta: None,
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
            meta: None,
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
            meta: None,
        }
    }

    pub fn is_macro(&self) -> bool {
        matches!(self.kind, FormKind::UserFn { is_macro: true, .. })
    }

    pub fn is_user_fn(&self) -> bool {
        matches!(self.kind, FormKind::UserFn { is_macro: false, .. })
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
    HashMap(HashMap<Form, Form>),
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

// impl FormKind {
//     fn mal_eq(&self, other: &FormKind) -> bool {
//         match (self, other) {
//             (FormKind::Nil, FormKind::Nil) => true,
//             (FormKind::Boolean(a), FormKind::Boolean(b)) => *a == *b,
//             (FormKind::Symbol(a), FormKind::Symbol(b)) => *a == *b,
//             (FormKind::Integer(a), FormKind::Integer(b)) => *a == *b,
//             (FormKind::Float(a), FormKind::Float(b)) => *a == *b,
//             (FormKind::String(a), FormKind::String(b)) => *a == *b,
//             (FormKind::Keyword(a), FormKind::Keyword(b)) => *a == *b,
//             (FormKind::List(a) | FormKind::Vector(a), FormKind::List(b) | FormKind::Vector(b)) => {
//                 *a == *b
//             }
//             (FormKind::HashMap(a), FormKind::HashMap(b)) => *a == *b,
//             (FormKind::NativeFn(_), _) => false,
//             (FormKind::UserFn { .. }, _) => false,
//             (_, _) => false,
//         }
//     }
// }

impl PartialEq for FormKind {
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

impl Eq for FormKind {}

impl Hash for FormKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(0x00);
        match self {
            FormKind::Nil => {
                state.write_u8(0x00);
                Hash::hash(&(), state);
            }
            FormKind::Boolean(x) => {
                state.write_u8(0x01);
                Hash::hash(x, state);
            }
            FormKind::Symbol(Ident { name }) => {
                state.write_u8(0x02);
                Hash::hash(name, state);
            }
            FormKind::Integer(x) => {
                state.write_u8(0x03);
                Hash::hash(x, state);
            }
            FormKind::Float(x) => {
                state.write_u8(0x04);
                Hash::hash(&x.to_bits(), state);
            }
            FormKind::String(x) => {
                state.write_u8(0x05);
                Hash::hash(x, state);
            }
            FormKind::Keyword(x) => {
                state.write_u8(0x06);
                Hash::hash(x, state);
            }
            FormKind::List(x) => {
                state.write_u8(0x07);
                x.iter().for_each(|v| {
                    state.write_u8(0x00);
                    Hash::hash(v, state);
                });
            }
            FormKind::Vector(x) => {
                state.write_u8(0x08);
                x.iter().for_each(|v| {
                    state.write_u8(0x00);
                    Hash::hash(v, state);
                });
            }
            FormKind::HashMap(x) => {
                state.write_u8(0x09);
                x.iter().for_each(|(k, v)| {
                    state.write_u8(0x00);
                    Hash::hash(k, state);
                    state.write_u8(0x01);
                    Hash::hash(v, state);
                });
            }
            FormKind::NativeFn(_) => {
                state.write_u8(0x0A);
                Hash::hash(&(), state);
            }
            FormKind::UserFn { .. } => {
                state.write_u8(0x0B);
                Hash::hash(&(), state);
            }
            FormKind::Atom(_) => {
                state.write_u8(0x0C);
                Hash::hash(&(), state);
            }
        }
    }
}
