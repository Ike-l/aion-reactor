use std::{pin::Pin, sync::Arc};

use crate::prelude::{Memory, ResourceId};

pub mod processors;
pub mod managers;

pub trait KernelSystem {
    fn init(&mut self, memory: &Memory) -> ResourceId;
    /// Cancel *Unsafe*
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
}

pub type StoredKernelSystem = Box<dyn KernelSystem>;