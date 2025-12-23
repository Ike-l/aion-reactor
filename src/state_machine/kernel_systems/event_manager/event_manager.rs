use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::kernel_systems::{KernelSystem, event_manager::{current_events::CurrentEvents, event_mapper::EventMapper, next_events::NextEvents}}};

pub struct EventManager;

impl KernelSystem for EventManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, EventMapper::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, NextEvents::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, CurrentEvents::default()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelEventManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, ) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let event_mapper = memory.resolve::<Shared<EventMapper>>(None, None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);
            next_events.extend(event_mapper.tick(&current_events).into_iter().map(|e| e.clone()));
        })
    }
}