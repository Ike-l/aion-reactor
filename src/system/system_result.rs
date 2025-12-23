use crate::state_machine::kernel_systems::{blocker_manager::prelude::Blocker, event_manager::event::Event};

#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
    WithEvent(Event),
    WithBlocker(Blocker)
}

#[derive(Debug)]
pub enum SystemResult {
    Events(Vec<SystemEvent>),
    Error(anyhow::Error),
    Conditional(bool)
}