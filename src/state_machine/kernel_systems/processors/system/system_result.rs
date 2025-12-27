use crate::prelude::{Blocker, EventId, NextBlockers, NextEvents, SystemId};


#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
    WithEvent(EventId),
    WithBlocker(Blocker)
}

impl SystemEvent {
    pub fn act(self, system_id: &SystemId, next_events: &mut NextEvents, next_blockers: &mut NextBlockers) {
        match self {
            SystemEvent::NoEvent => { next_events.remove(&EventId::from(system_id.clone().into_id())); },
            SystemEvent::WithEvent(event) => { next_events.insert(event); },
            SystemEvent::WithBlocker(blocker) => { next_blockers.insert(blocker); },
        }
    }
}

#[derive(Debug)]
pub enum SystemResult {
    Events(Vec<SystemEvent>),
    Event(SystemEvent),

    Error(anyhow::Error),
    Conditional(bool)
}