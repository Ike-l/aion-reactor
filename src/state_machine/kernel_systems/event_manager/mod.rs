use std::{collections::{HashMap, HashSet}, pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}, state_machine::{kernel_systems::{event_manager::event::{CurrentEvents, Event, NextEvents}, KernelSystem}, transition_phases::TransitionPhase}};

pub mod event;

pub struct EventManager;

pub struct EventMapper(HashMap<Event, HashSet<Event>>);

impl EventMapper {
    pub fn insert(&mut self, from: Event, to: Event) -> bool {
        self.0.entry(from).or_default().insert(to)
    }
}

impl KernelSystem for EventManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, EventMapper(HashMap::new())).unwrap().is_ok());
        assert!(memory.insert(None, None, None, NextEvents::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, CurrentEvents::default()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelEventManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            next_events.insert(phase);

            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);

            let new_events = memory.resolve::<Shared<EventMapper>>(None, None, None, None).unwrap().unwrap();

            for event in current_events.read() {
                if let Some(new_events) = new_events.0.get(event) {
                    for new_event in new_events {
                        next_events.insert(new_event.clone());
                    }
                }
            }
        })
    }
}