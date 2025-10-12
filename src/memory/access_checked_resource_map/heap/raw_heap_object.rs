use std::cell::UnsafeCell;

#[derive(Debug, Default)]
pub struct RawHeapObject<T>(UnsafeCell<T>);

unsafe impl<T> Send for RawHeapObject<T> {}
unsafe impl<T> Sync for RawHeapObject<T> {}

impl<T> RawHeapObject<T> {
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