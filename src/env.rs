use std::{collections::HashMap, rc::Rc};

use crate::{Error, Form, Result};

#[derive(Clone, Debug)]
struct EnvInner {
    data: HashMap<String, Form>,
    parent: Option<Env>,
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
                parent: Some(parent.clone()),
            }),
        }
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: Form) {
        Rc::make_mut(&mut self.inner)
            .data
            .insert(key.as_ref().into(), value);
    }

    pub fn get(&self, key: &str) -> Result<Form> {
        let mut current = self;
        loop {
            if let Some(value) = (*current.inner).data.get(key) {
                return Ok(value.clone());
            } else if let Some(ref parent) = (*current.inner).parent {
                current = parent;
            } else {
                return Err(Error::UnknownSymbol(key.into()));
            }
        }
    }
}
