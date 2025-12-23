use std::collections::HashSet;

use crate::state_machine::kernel_systems::blocker_manager::{blocker::Blocker, next_blockers::NextBlockers};

#[derive(Debug, Default, Clone)]
pub struct CurrentBlockers(HashSet<Blocker>);

impl CurrentBlockers {
    pub fn tick(&mut self, new_blockers: &mut NextBlockers) {
        self.0.clear();

        self.0.extend(new_blockers.drain());
    }

    pub fn read(&self) -> impl Iterator<Item = &Blocker> {
        self.0.iter()
    }

    pub fn blocks(&self, id: Blocker) -> bool {
        self.0.contains(&id)
    }
}