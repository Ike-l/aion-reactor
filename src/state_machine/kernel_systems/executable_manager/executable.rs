use crate::state_machine::kernel_systems::event_manager::event::Event;

#[derive(Clone)]
pub struct Executable {
    pub label: String,
    pub trigger: Event,
}

impl Executable {
    pub fn new(label: String, trigger: Event) -> Self {
        Self {
            label,
            trigger
        }
    }
}
