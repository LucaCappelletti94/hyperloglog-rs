//! Submodule providing utilities for parallel testing.

use std::cell::UnsafeCell;

pub struct ThreadUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for ThreadUnsafeCell<T> {}
unsafe impl<T> Send for ThreadUnsafeCell<T> {}

impl<T> From<T> for ThreadUnsafeCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> ThreadUnsafeCell<T> {
    pub fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}