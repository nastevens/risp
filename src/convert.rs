use crate::{
    form::{Atom, Ident},
    Form, FormKind, Result,
};

#[derive(Debug)]
pub struct Rest {
    pub(crate) values: Vec<Form>,
}

impl Rest {
    pub fn new(iter: impl IntoIterator<Item = Form>) -> Rest {
        Rest {
            values: iter.into_iter().collect(),
        }
    }
}

impl<T, E> TryInto<Vec<T>> for Form
where
    Form: TryInto<T, Error = E>,
    crate::Error: From<E>,
{
    type Error = crate::Error;

    fn try_into(self) -> Result<Vec<T>> {
        match self.kind {
            FormKind::Nil => Ok(Vec::new()),
            FormKind::List(inner) | FormKind::Vector(inner) => {
                inner.into_iter().map(|x| Ok(x.try_into()?)).collect()
            }
            _ => Err(crate::Error::InvalidArgument),
        }
    }
}

impl From<Form> for () {
    fn from(_val: Form) -> Self {}
}

impl From<Form> for bool {
    fn from(val: Form) -> Self {
        !matches!(val.kind, FormKind::Nil | FormKind::Boolean(false))
    }
}

impl From<Option<Form>> for Form {
    fn from(val: Option<Form>) -> Self {
        match val {
            Some(x) => x,
            None => Form::nil(),
        }
    }
}

impl TryInto<i64> for Form {
    type Error = crate::Error;

    fn try_into(self) -> Result<i64> {
        match self.kind {
            FormKind::Integer(i) => Ok(i),
            FormKind::Float(f) => Ok(f as i64),
            _ => Err(crate::Error::InvalidArgument),
        }
    }
}

impl TryInto<Ident> for Form {
    type Error = crate::Error;

    fn try_into(self) -> std::result::Result<Ident, Self::Error> {
        match self.kind {
            FormKind::Symbol(ident) => Ok(ident),
            _ => Err(crate::Error::InvalidArgument),
        }
    }
}

impl TryInto<String> for Form {
    type Error = crate::Error;

    fn try_into(self) -> std::result::Result<String, Self::Error> {
        match self.kind {
            FormKind::String(s) => Ok(s),
            _ => Err(crate::Error::InvalidArgument),
        }
    }
}

impl TryInto<Atom> for Form {
    type Error = crate::Error;

    fn try_into(self) -> std::result::Result<Atom, Self::Error> {
        match self.kind {
            FormKind::Atom(atom) => Ok(atom),
            _ => Err(crate::Error::InvalidArgument),
        }
    }
}

macro_rules! tuple_impls {
    ($($len:tt => ($($name:ident $error:ident)+))+) => {
        $(
            impl<$($name, $error),+,> TryInto<($($name,)+)> for Form
            where
                $(
                    Form: TryInto<$name, Error = $error>,
                    crate::Error: From<$error>,
                    $name: 'static,
                )+
            {
                type Error = crate::Error;

                #[allow(non_snake_case)]
                fn try_into(self) -> std::result::Result<($($name,)+), crate::Error> {
                    match self.kind {
                        crate::form::FormKind::List(mut inner) => {
                            let mut iter = inner.drain(..).fuse();
                            $(
                                let $name = Into::<Form>::into(iter.next()).try_into()?;
                            )+
                            if iter.next().is_none() {
                                Ok(($($name,)+))
                            } else {
                                Err(crate::Error::InvalidArgument)
                            }
                        }
                        _ => Err(crate::Error::InvalidArgument),
                    }
                }
            }

            impl<$($name, $error),+,> TryInto<($($name,)+ Rest)> for Form
            where
                $(
                    Form: TryInto<$name, Error = $error>,
                    crate::Error: From<$error>,
                    $name: 'static,
                )+
            {
                type Error = crate::Error;

                #[allow(non_snake_case)]
                fn try_into(self) -> std::result::Result<($($name,)+ Rest), crate::Error> {
                    match self.kind {
                        crate::form::FormKind::List(mut inner) => {
                            let mut iter = inner.drain(..).fuse();
                            $(
                                let $name = Into::<Form>::into(iter.next()).try_into()?;
                            )+
                            Ok(($($name,)+ Rest::new(iter)))
                        }
                        _ => Err(crate::Error::InvalidArgument),
                    }
                }
            }
        )+
    }
}

tuple_impls! {
    1 => (T0 E0)
    2 => (T0 E0 T1 E1)
    3 => (T0 E0 T1 E1 T2 E2)
    4 => (T0 E0 T1 E1 T2 E2 T3 E3)
    5 => (T0 E0 T1 E1 T2 E2 T3 E3 T4 E4)
}
