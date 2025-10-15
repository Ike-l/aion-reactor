use std::{pin::Pin, sync::Arc};

use crate::{injection::injection_primitives::unique::Unique, memory::Memory, state_machine::kernel_systems::{blocker_manager::blocker::{CurrentBlockers, NextBlockers}, KernelSystem}};

pub mod blocker;

pub struct BlockerManager;

impl BlockerManager {
    pub fn new() -> Self {
       Self 
    }
}

impl KernelSystem for BlockerManager {
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextBlockers>>(None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentBlockers>>(None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);
        })
    }
}