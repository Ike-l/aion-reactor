use std::{pin::Pin, sync::Arc};

use crate::{memory::Memory, state_machine::{kernel_systems::KernelSystem, transition_phases::TransitionPhase}};

pub struct BackgroundProcessor;

impl BackgroundProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl KernelSystem for BackgroundProcessor {
    fn tick(&mut self, _memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        todo!()

        // Start threads
        // Collect threads
    }
}