use serde::de::{value::SeqDeserializer, Deserialize, Deserializer, IntoDeserializer};

use crate::{ast::Ident, Env, Error, Form, FormKind, Result};

impl<'de> Deserialize<'de> for Form {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Unexpected, Visitor};

        struct FormVisitor;

        impl<'de> Visitor<'de> for FormVisitor {
            type Value = Form;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid Risp Form")
            }

            fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::boolean(v))
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::int(v))
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                let as_i64 = i64::try_from(v).map_err(|_| {
                    Error::invalid_value(Unexpected::Unsigned(v), &"an integer of size i64")
                })?;
                Ok(Form::int(as_i64))
            }

            fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::float(v))
            }

            fn visit_char<E>(self, v: char) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Err(Error::invalid_type(
                    Unexpected::Char(v),
                    &"a Form type (char is not supported)",
                ))
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_string(String::from(v))
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::string(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Err(Error::invalid_type(
                    Unexpected::Bytes(v),
                    &"a Form type (bytes are not supported)",
                ))
            }

            fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::nil())
            }

            fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Form::nil())
            }

            fn visit_newtype_struct<D>(
                self,
                deserializer: D,
            ) -> std::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                Ok(Form::list(vec))
            }

            fn visit_map<A>(self, _map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                Err(Error::invalid_type(Unexpected::Map, &self))
            }

            fn visit_enum<A>(self, _data: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                Err(Error::invalid_type(
                    Unexpected::Enum,
                    &"a Form type (enums are not supported)",
                ))
            }
        }

        deserializer.deserialize_any(FormVisitor)
    }
}

impl serde::de::Error for crate::Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        crate::Error::SerdeError(msg.to_string())
    }
}

impl<'de> IntoDeserializer<'de, Error> for Form {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

fn visit_array<'de, V>(list: Vec<Form>, visitor: V) -> std::result::Result<V::Value, Error>
where
    V: serde::de::Visitor<'de>,
{
    let mut deserializer = SeqDeserializer::new(list.into_iter());
    visitor.visit_seq(&mut deserializer)
}

impl<'de> Deserializer<'de> for Form {
    type Error = crate::Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.kind {
            FormKind::Nil => visitor.visit_unit(),
            FormKind::Boolean(b) => visitor.visit_bool(b),
            FormKind::Symbol(_) => todo!(),
            FormKind::Integer(i) => visitor.visit_i64(i),
            FormKind::Float(f) => visitor.visit_f64(f),
            FormKind::String(s) => visitor.visit_string(s),
            FormKind::Keyword(_) => todo!(),
            FormKind::List(inner) => visit_array(inner, visitor),
            FormKind::Vector(inner) => visit_array(inner, visitor),
            FormKind::HashMap(_) => todo!(),
            FormKind::NativeFn { name: _, f: _ } => todo!(),
            FormKind::Fn => todo!(),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub fn eval_ast(form: Form, env: &mut Env) -> Result<Form> {
    match form {
        Form {
            kind: FormKind::Symbol(Ident { name }),
        } => env.get(&name),
        Form {
            kind: FormKind::List(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::list(evaluated))
        }
        Form {
            kind: FormKind::Vector(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::vector(evaluated))
        }
        Form {
            kind: FormKind::HashMap(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::hash_map(evaluated))
        }
        other => return Ok(other),
    }
}

fn fn_name<'a>(form: &'a Form) -> Option<&'a str> {
    match form.kind {
        FormKind::List(ref inner) => {
            if let Some(Form {
                kind: FormKind::Symbol(Ident { name }),
            }) = inner.first()
            {
                Some(name)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn eval(form: Form, env: &mut Env) -> Result<Form> {
    // match fn_name(&form) {
    //     Some("def!") => todo!(),
    //     Some("let*") => todo!(),
    //     Some(other) => todo!(),
    //     None => todo!(),
    // }

    let (called_as, evaluated) = match form.kind {
        FormKind::List(ref list) if !list.is_empty() => match list.get(0) {
            Some(Form {
                kind: FormKind::Symbol(Ident { name }),
            }) => (name.to_string(), eval_ast(form, env)?),
            _ => ("#<anonymous>".to_string(), eval_ast(form, env)?),
        },
        _ => return eval_ast(form, env),
    };

    match evaluated.kind {
        FormKind::List(mut list) => {
            if list.len() < 1 {
                return Err(Error::InvalidArgument);
            }

            let args = list.drain(1..).collect::<Vec<Form>>();
            if let FormKind::NativeFn { f, .. } = list.remove(0).kind {
                f(&called_as, Form::list(args))
            } else {
                Err(Error::InvalidApply)
            }
        }
        _ => todo!(),
    }
}
