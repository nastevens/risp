use std::{collections::HashMap, rc::Rc};

use crate::{Error, Form, Result};

#[derive(Debug)]
struct EnvInner {
    data: HashMap<String, Form>,
    parent: Option<Rc<EnvInner>>,
}

#[derive(Clone, Debug)]
pub struct Env {
    inner: Rc<EnvInner>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            inner: Rc::new(EnvInner {
                data: HashMap::new(),
                parent: None,
            }),
        }
    }

    pub fn new_with(parent: &Env) -> Env {
        Env {
            inner: Rc::new(EnvInner {
                data: HashMap::new(),
                parent: Some(parent.inner.clone()),
            })
        }
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: Form) {
        Rc::get_mut(&mut self.inner)
            .unwrap()
            .data
            .insert(key.as_ref().into(), value);
    }

    pub fn get(&self, key: &str) -> Result<Form> {
        let mut current = &self.inner;
        loop {
            if let Some(value) = (*current).data.get(key) {
                return Ok(value.clone())
            } else if let Some(ref parent) = (*current).parent {
                current = parent;
            } else {
                return Err(Error::UnknownSymbol(key.into()))
            }

        }
    }
}
