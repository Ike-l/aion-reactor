use std::collections::HashMap;

pub use crate::kernel_prelude::*;
use crate::{id::Id, memory::access_checked_heap::heap::HeapId, state_machine::kernel_systems::event_manager::event::Event};

pub struct ExecutableManager;

pub struct ExecutableRegistry(pub HashMap<String, Executable>);
pub struct Executable {
    label: String,
    trigger: Event,
}

impl KernelSystem for ExecutableManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        ResourceId::Heap(HeapId::Label(Id("KernelExecutableManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);

        Box::pin(async move {
            
        })
    }
}