use std::collections::HashSet;

use crate::state_machine::kernel_systems::blocker_manager::blocker::Blocker;

#[derive(Debug, Default)]
pub struct NextBlockers(HashSet<Blocker>);

impl NextBlockers {
    pub fn insert(&mut self, blocker: Blocker) -> bool {
        self.0.insert(blocker)
    }

    pub fn remove(&mut self, blocker: &Blocker) -> bool {
        self.0.remove(blocker)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Blocker> {
        self.0.drain()
    }
}