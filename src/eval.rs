use serde::de::{Deserialize, Deserializer, IntoDeserializer, Unexpected, Expected};

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

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> std::result::Result<V::Value, Error>
        where
            V: serde::de::Visitor<'de>,
        {
            match self.kind {
                FormKind::Integer(n) => n.into_deserializer().$method(visitor),
                FormKind::Float(n) => n.into_deserializer().$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

impl Form {
    fn invalid_type<E>(&self, expected: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), expected)
    }

    fn unexpected(&self) -> Unexpected {
        match self.kind {
            FormKind::Nil => Unexpected::Unit,
            FormKind::Boolean(b) => Unexpected::Bool(b),
            FormKind::Symbol(_) => todo!(),
            FormKind::Integer(_) => todo!(),
            FormKind::Float(_) => todo!(),
            FormKind::String(_) => todo!(),
            FormKind::Keyword(_) => todo!(),
            FormKind::List(_) => todo!(),
            FormKind::Vector(_) => todo!(),
            FormKind::HashMap(_) => todo!(),
            FormKind::NativeFn { name, f } => todo!(),
            FormKind::Fn => todo!(),
        }
    }
}

impl<'de> Deserializer<'de> for Form {
    type Error = crate::Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.kind {
            FormKind::Nil => todo!(),
            FormKind::Boolean(_) => todo!(),
            FormKind::Symbol(_) => todo!(),
            FormKind::Integer(_) => todo!(),
            FormKind::Float(_) => todo!(),
            FormKind::String(_) => todo!(),
            FormKind::Keyword(_) => todo!(),
            FormKind::List(_) => todo!(),
            FormKind::Vector(_) => todo!(),
            FormKind::HashMap(_) => todo!(),
            FormKind::NativeFn { name, f } => todo!(),
            FormKind::Fn => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.kind {
            FormKind::Boolean(b) => visitor.visit_bool(b),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

pub fn eval_ast(form: Form, env: &mut Env) -> Result<Form> {
    match form {
        Form {
            kind: FormKind::Symbol(Ident { name }),
        } => env.get(&name),
        Form {
            kind: FormKind::List(list) | FormKind::Vector(list),
        } => {
            let vec = list
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::list(vec))
        }
        other => return Ok(other),
    }
}

// fn apply() {}

pub fn eval(form: Form, env: &mut Env) -> Result<Form> {
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
