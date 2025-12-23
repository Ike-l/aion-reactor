use crate::state_machine::kernel_systems::event_manager::event::Event;

pub struct RegisteredDelay {
    pub activated_by: Event,
    pub then_inserts: Event,
    pub delayed_by: Option<Event>
}