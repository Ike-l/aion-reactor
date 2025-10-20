use std::{pin::Pin, sync::Arc};

use crate::{memory::Memory, state_machine::{kernel_systems::KernelSystem, transition_phases::TransitionPhase}};

pub struct FinishBackgroundProcessor;

impl FinishBackgroundProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl KernelSystem for FinishBackgroundProcessor {
    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        // get join handles (async & sync)
        // if handle finished then insert next event 
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            // let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None).unwrap().unwrap();
            // let systems = Processor::get_systems(&memory, &system_registry.0);
            
            // for (id, system_metadata) in systems {
                // try reserve all accesses concurrently
                // if fail then panic
                // else run all either: async run in tokio spawn task, sync run in separate threads.
                // store join handles for finish
            // }
        })
    }
}