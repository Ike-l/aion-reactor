use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::kernel_systems::{KernelSystem, delay_manager::{delay_buffer::DelayBuffer, delay_registry::DelayRegistry}, event_manager::prelude::CurrentEvents}};

pub struct DelayManager;

impl KernelSystem for DelayManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, DelayRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, DelayBuffer::default()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelDelayManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut buffer = memory.resolve::<Unique<DelayBuffer>>(None, None, None, None).unwrap().unwrap();
            let registry = memory.resolve::<Shared<DelayRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            
            buffer.tick(&registry, &mut current_events);
        })
    }
}