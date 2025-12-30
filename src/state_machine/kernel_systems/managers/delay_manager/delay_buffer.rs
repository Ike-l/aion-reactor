use std::collections::HashSet;

use crate::prelude::{CurrentEvents, Delay, DelayRegistry, EventId, NextEvents};

#[derive(Default)]
pub struct DelayBuffer(Vec<Delay>);

impl DelayBuffer {
    pub fn stage_activatable(&mut self, delay_registry: &DelayRegistry, current_events: &CurrentEvents) {
        self.0.extend(delay_registry.get_activatable(current_events));
    }

    pub fn get_delays(&self) -> impl Iterator<Item = EventId> {
        self.0.iter().filter_map(|delay| Some(delay.delayed_by.clone()?) )
    }

    pub fn tick(&mut self, delay_registry: &DelayRegistry, current_events: &CurrentEvents, next_events: &mut NextEvents) {
        // for each registered delay, if current events contains the activation event (`from`), become activated
        self.stage_activatable(&delay_registry, &current_events);
    
        let current_delays = self.get_delays().collect::<HashSet<_>>();
        
        for delay in self.0.drain(..).collect::<Vec<_>>() {
            match delay.delayed_by.as_ref() {
                Some(delayed_by) if current_delays.contains(&delayed_by) => { self.0.push(delay); },
                _ => { next_events.insert(delay.then_inserts); },
            };
        }
    }
}

