use std::ops::Deref;

/// An owned smart pointer.
pub struct P<T: ?Sized> {
    ptr: Box<T>,
}

#[allow(non_snake_case)]
pub fn P<T: 'static>(value: T) -> P<T> {
    P {
        ptr: Box::new(value),
    }
}

impl<T: 'static> P<T> {
    pub fn into_inner(self) -> T {
        *self.ptr
    }

    pub fn map<F>(mut self, f: F) -> P<T>
    where
        F: FnOnce(T) -> T,
    {
        let x = f(*self.ptr);
        *self.ptr = x;
        self
    }
}

impl<T: ?Sized> Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.ptr
    }
}

impl<T: 'static + Clone> Clone for P<T> {
    fn clone(&self) -> P<T> {
        P((**self).clone())
    }
}
