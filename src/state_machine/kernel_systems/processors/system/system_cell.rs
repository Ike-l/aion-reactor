use std::cell::UnsafeCell;

use crate::prelude::System;


#[derive(Debug)]
pub struct SystemCell(UnsafeCell<System>);

impl SystemCell {
    pub fn new(system: System) -> Self {
        Self(UnsafeCell::new(system))
    }

    pub fn consume(self) -> System {
        self.0.into_inner()
    }

    /// Safety:
    /// Ensure only 1 reference to System
    pub unsafe fn get(&self) -> &mut System {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl Send for SystemCell {}
unsafe impl Sync for SystemCell {}