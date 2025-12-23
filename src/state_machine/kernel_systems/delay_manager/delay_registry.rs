use crate::state_machine::kernel_systems::{delay_manager::delay::{Delay, registered_delay::RegisteredDelay}, event_manager::prelude::CurrentEvents};

#[derive(Default)]
pub struct DelayRegistry(Vec<RegisteredDelay>);

impl DelayRegistry {
    pub fn push(&mut self, registered_delay: RegisteredDelay) {
        self.0.push(registered_delay);
    }

    pub fn get_activatable(&self, current_events: &CurrentEvents) -> impl Iterator<Item = Delay> {
        self.0.iter().filter_map(|registered_delay| {
            if current_events.contains(&registered_delay.activated_by) {
                return Some(Delay::new(registered_delay.then_inserts.clone(), registered_delay.delayed_by.clone()));
            }

            None
        })
    }
}