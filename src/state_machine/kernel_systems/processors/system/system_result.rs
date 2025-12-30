use tracing::{Level, event};

use crate::prelude::{BlockerId, EventId, NextBlockers, NextEvents, SystemEventRegistry, SystemId};


#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
    WithEvent(EventId),
    WithBlocker(BlockerId)
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
    #[deprecated(note = "Use SystemResult::Events")]
    // true => inserts events via a registry
    // false => removes the current system as an event (SystemEvent::NoEvent)
    Conditional(bool)
}

impl SystemResult {
    pub fn act(
        self, 
        system_id: &SystemId, 
        mut next_events: &mut NextEvents, 
        mut next_blockers: &mut NextBlockers,
        system_event_registry: &SystemEventRegistry,
        parent_span: tracing::Span,
    ) {
        match self {
            SystemResult::Event(system_event) => {
                system_event.act(&system_id, &mut next_events, &mut next_blockers);
            }
            SystemResult::Events(system_events) => {
                for system_event in system_events {
                    system_event.act(&system_id, &mut next_events, &mut next_blockers);
                }
            },
            SystemResult::Error(error) => event!(parent: parent_span, Level::ERROR, system_result_error=%error),
            #[allow(deprecated)]
            SystemResult::Conditional(bool) => {
                if bool {
                    if let Some(events) = system_event_registry.get(&system_id) {
                        next_events.extend(events.clone().into_iter());
                    } else {
                        event!(parent: parent_span, Level::WARN, "No Events in SystemEventRegistry");
                    }
                } else {
                    SystemEvent::NoEvent.act(system_id, next_events, next_blockers);
                }
            }
        }
    }

    // match result {
    //                 SystemResult::Event(system_event) => {
    //                     system_event.act(&system_id, &mut next_events, &mut next_blockers);
    //                 }
    //                 SystemResult::Events(system_events) => {
    //                     for system_event in system_events {
    //                         system_event.act(&system_id, &mut next_events, &mut next_blockers);
    //                     }
    //                 },
    //                 SystemResult::Error(error) => event!(Level::ERROR, system_result_error=%error),
    //                 SystemResult::Conditional(bool) => {
    //                     if bool {
    //                         if let Some(events) = system_event_registry.get(&system_id) {
    //                             next_events.extend(events.clone().into_iter());
    //                         } else {
    //                             event!(Level::WARN, "No Events in SystemEventRegistry");
    //                         }
    //                     }
    //                 }
    //             }
}