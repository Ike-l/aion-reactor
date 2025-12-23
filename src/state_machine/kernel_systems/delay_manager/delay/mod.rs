use crate::state_machine::kernel_systems::event_manager::event::Event;

pub mod registered_delay;

pub struct Delay {
    pub then_inserts: Event,
    pub delayed_by: Option<Event>
}

impl Delay {
    pub fn new(then_inserts: Event, delayed_by: Option<Event>) -> Self {
        Self { then_inserts, delayed_by }
    }
}
