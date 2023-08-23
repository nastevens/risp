use crate::{Env, Result};

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn from_str(name: &str) -> Ident {
        Ident { name: name.into() }
    }
}

#[derive(Clone, Debug)]
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
    form_predicate_fn!(is_list, FormKind::List(_));

    pub fn is_empty_list(&self) -> bool {
        matches!(self.kind, FormKind::List(ref inner) if inner.is_empty())
    }

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

    pub fn user_fn(binds: Vec<Ident>, body: Form, env: Env) -> Form {
        Form {
            kind: FormKind::UserFn {
                binds,
                bind_rest: None,
                body: Box::new(body),
                env,
            },
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
    },
}

impl std::fmt::Debug for FormKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FormKind::")?;
        match self {
            FormKind::Nil => f.write_str("Nil"),
            FormKind::Boolean(val) => write!(f, "Boolean({:?})", val),
            FormKind::Symbol(val) => write!(f, "Symbol({:?})", val),
            FormKind::Integer(val) => write!(f, "Integer({:?})", val),
            FormKind::Float(val) => write!(f, "Float({:?})", val),
            FormKind::String(val) => write!(f, "String({:?})", val),
            FormKind::Keyword(val) => write!(f, "Keyword({:?})", val),
            FormKind::List(val) => write!(f, "List({:?})", val),
            FormKind::Vector(val) => write!(f, "Vector({:?})", val),
            FormKind::HashMap(val) => write!(f, "HashMap({:?})", val),
            FormKind::NativeFn(_) => write!(f, "NativeFn(#<function>)"),
            FormKind::UserFn { .. } => write!(f, "UserFn"),
        }
    }
}

//     pub fn expand(&self) -> Result<String, Error> {
//         use nom::{
//             branch::alt,
//             bytes::complete::{escaped_transform, is_not, tag},
//             combinator::value,
//             sequence::delimited,
//             IResult,
//         };
//         fn extract_and_expand(input: &str) -> IResult<&str, String> {
//             alt((
//                 value(String::new(), tag("\"\"")),
//                 delimited(
//                     tag("\""),
//                     escaped_transform(
//                         is_not("\\\""),
//                         '\\',
//                         alt((
//                             value("\\", tag("\\")),
//                             value("\"", tag("\"")),
//                             value("\n", tag("n")),
//                         )),
//                     ),
//                     tag("\""),
//                 ),
//             ))(input)
//         }
//         Ok(extract_and_expand(&self.value)
//             .map(|result| result.1)
//             .map_err(|_| Error::Eof)?)
//     }
// }
