use std::{collections::HashMap, rc::Rc};

use crate::{Error, Form, Result};

struct EnvInner {
    data: HashMap<String, Form>,
}

pub struct Env {
    outer: Option<Rc<Env>>,
    inner: Rc<EnvInner>,
}

impl Env {
    pub fn new() -> Env {
        let mut env = Env {
            outer: None,
            inner: Rc::new(EnvInner {
                data: HashMap::new(),
            }),
        };
        crate::core::populate(&mut env);
        env
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: Form) {
        Rc::get_mut(&mut self.inner)
            .unwrap()
            .data
            .insert(key.as_ref().into(), value);
    }

    pub fn get(&self, key: &str) -> Result<Form> {
        if let Some(value) = (*self.inner).data.get(key) {
            Ok(value.clone())
        } else if let Some(ref outer) = self.outer {
            (*outer).get(key)
        } else {
            Err(Error::UnknownSymbol(key.into()))
        }
    }
}
