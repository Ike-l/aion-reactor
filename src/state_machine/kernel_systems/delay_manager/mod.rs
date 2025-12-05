use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::{kernel_systems::{KernelSystem, event_manager::event::{CurrentEvents, Event}}, }};

pub struct DelayManager;

// From | Into | Delaying
pub struct DelayRegistry(pub Vec<(Event, Event, Option<Event>)>);

// Into | Delaying
pub struct DelayBuffer(pub Vec<(Event, Option<Event>)>);

impl KernelSystem for DelayManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, DelayRegistry(Vec::new())).unwrap().is_ok());
        assert!(memory.insert(None, None, None, DelayBuffer(Vec::new())).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelDelayManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_buffer = DelayBuffer(Vec::new());

            let mut buffer = memory.resolve::<Unique<DelayBuffer>>(None, None, None, None).unwrap().unwrap();
            let registry = memory.resolve::<Shared<DelayRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();

            // load buffer with registry that "could" be activated
            for (from, into, delay) in registry.0.iter() {
                if current_events.contains(from) {
                    buffer.0.push((into.clone(), delay.clone()));
                }
            }

            let new_delays = buffer.0.iter().filter_map(|(_, delay)| Some(delay.clone()?) ).collect::<Vec<_>>();

            // if delayed then queue for next time else put in current event
            for (into, delay) in buffer.0.drain(..) {
                if new_delays.contains(&into) {
                    next_buffer.0.push((into, delay));
                } else {
                    current_events.insert(into);
                }
            }

            buffer.0.extend(next_buffer.0);
        })
    }
}