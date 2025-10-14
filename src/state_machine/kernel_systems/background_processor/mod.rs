use std::{pin::Pin, sync::Arc};

use crate::{memory::Memory, state_machine::kernel_systems::KernelSystem};

pub struct BackgroundProcessor;

impl BackgroundProcessor {
    pub fn new() -> Self {
        BackgroundProcessor
    }
}

impl KernelSystem for BackgroundProcessor {
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        todo!()
    }
}