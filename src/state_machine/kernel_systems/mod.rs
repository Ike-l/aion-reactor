use std::{pin::Pin, sync::Arc};

use crate::{memory::{Memory, ResourceId}};

pub mod processor;
pub mod event_manager;
pub mod blocker_manager;
pub mod background_processor;
pub mod delay_manager;
pub mod executable_manager;
pub mod read_only_processor;

pub trait KernelSystem {
    fn init(&mut self, memory: &Memory) -> ResourceId;
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;