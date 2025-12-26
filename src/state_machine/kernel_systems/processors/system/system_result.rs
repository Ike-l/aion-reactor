use crate::prelude::{Blocker, EventId};


#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
    WithEvent(EventId),
    WithBlocker(Blocker)
}

#[derive(Debug)]
pub enum SystemResult {
    Events(Vec<SystemEvent>),
    Error(anyhow::Error),
    Conditional(bool)
}