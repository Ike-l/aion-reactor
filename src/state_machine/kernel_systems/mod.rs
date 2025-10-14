use std::{pin::Pin, sync::Arc};

use crate::memory::Memory;

pub mod processor;
pub mod event_manager;
pub mod blocker_manager;
pub mod background_processor;

pub trait KernelSystem {
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;