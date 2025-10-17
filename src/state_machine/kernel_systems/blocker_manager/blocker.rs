use std::collections::HashSet;

use crate::id::Id;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Blocker(Id);

impl From<Id> for Blocker {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

impl Blocker {
    pub fn id(&self) -> &Id {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct NextBlockers(HashSet<Blocker>);

impl NextBlockers {
    pub fn insert<T: Into<Blocker>>(&mut self, blocker: T) -> bool {
        self.0.insert(blocker.into())
    }

    pub fn remove(&mut self, blocker: &Blocker) -> bool {
        self.0.remove(blocker)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CurrentBlockers(HashSet<Blocker>);

impl CurrentBlockers {
    pub fn tick(&mut self, new_blockers: &mut NextBlockers) {
        self.0.clear();

        self.0.extend(new_blockers.0.drain());
    }

    pub fn read(&self) -> impl Iterator<Item = &Blocker> {
        self.0.iter()
    }

    pub fn blocks<T: Into<Blocker>>(&self, id: T) -> bool {
        self.0.contains(&id.into())
    }
}