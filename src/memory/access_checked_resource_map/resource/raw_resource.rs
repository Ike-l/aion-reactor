use std::cell::UnsafeCell;

#[derive(Debug, Default)]
pub struct RawResource<T>(UnsafeCell<T>);

unsafe impl<T> Send for RawResource<T> {}
unsafe impl<T> Sync for RawResource<T> {}

impl<T> RawResource<T> {
    pub fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }

    pub fn get(&self) -> *mut T {
        self.0.get()
    }

    pub fn consume(self) -> T {
        self.0.into_inner()
    }
}