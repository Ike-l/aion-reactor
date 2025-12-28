use std::{collections::HashSet, ops::Range};

use crate::prelude::{BlockerId, NextBlockers};

#[derive(Debug, Default, Clone)]
pub struct CurrentBlockers(HashSet<BlockerId>);

impl CurrentBlockers {
    pub fn tick(&mut self, new_blockers: &mut NextBlockers) {
        self.0.clear();

        self.0.extend(new_blockers.drain());
    }

    pub fn read(&self) -> impl Iterator<Item = &BlockerId> {
        self.0.iter()
    }

    pub fn blocks(&self, blocker: &BlockerId) -> bool {
        self.0.contains(blocker)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// no guarantees about the ordering
    pub fn get_range(&self, amount: Range<usize>) -> impl Iterator<Item = &BlockerId> {
        self.0.iter().take(amount.end)
    }
}