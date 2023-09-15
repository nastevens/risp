use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crate::{Error, Form, Result};

#[derive(Clone, Debug)]
struct EnvInner {
    data: HashMap<String, Form>,
    parent: Option<Env>,
}

/// Env
///
/// Prefer the Extend implementation - it avoids repeatedly taking/releasing the mutex lock
#[derive(Clone, Debug)]
pub struct Env {
    inner: Rc<Mutex<EnvInner>>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            inner: Rc::new(Mutex::new(EnvInner {
                data: HashMap::new(),
                parent: None,
            })),
        }
    }

    pub fn new_with(parent: &Env) -> Env {
        Env {
            inner: Rc::new(Mutex::new(EnvInner {
                data: HashMap::new(),
                parent: Some(parent.clone()),
            })),
        }
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: Form) {
        self.inner
            .lock()
            .expect("Poisoned mutex")
            .data
            .insert(key.as_ref().into(), value);
    }

    /// Get the value assigned to `key`, or return an `UnknownSymbol` error
    pub fn get(&self, key: &str) -> Result<Form> {
        let guard = self.inner.lock().expect("Poisoned mutex");
        // Food
        if let Some(value) = guard.data.get(key) {
            Ok(value.clone())
        } else if let Some(ref parent) = guard.parent {
            parent.get(key)
        } else {
            Err(Error::UnknownSymbol(key.into()))
        }
    }

    /// Retrieve the root environment
    pub fn root(&self) -> Env {
        if let Some(ref parent) = self.inner.lock().expect("Poisoned mutex").parent {
            return parent.root();
        } else {
            return self.clone();
        }
    }
}

impl Extend<(String, Form)> for Env {
    fn extend<T: IntoIterator<Item = (String, Form)>>(&mut self, iter: T) {
        let data = &mut self.inner.lock().expect("Poisoned mutex").data;
        for elem in iter {
            data.insert(elem.0, elem.1);
        }
    }
}
