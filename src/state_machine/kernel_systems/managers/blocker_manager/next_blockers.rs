use std::collections::HashSet;

use crate::prelude::BlockerId;

#[derive(Debug, Default)]
pub struct NextBlockers(HashSet<BlockerId>);

impl NextBlockers {
    pub fn insert(&mut self, blocker: BlockerId) -> bool {
        self.0.insert(blocker)
    }

    pub fn remove(&mut self, blocker: &BlockerId) -> bool {
        self.0.remove(blocker)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = BlockerId> {
        self.0.drain()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}