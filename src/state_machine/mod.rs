use std::sync::Arc;

use crate::{id::Id, injection::{injection_primitives::unique::Unique, injection_trait::Injection}, memory::{errors::ResolveError, resource_id::Resource, Memory, ResourceId}, processor::Processor, state_machine::{blocker::{CurrentBlockers, NextBlockers}, event::{CurrentEvents, NextEvents}}};

pub mod event;
pub mod blocker;
pub mod blacklist;

#[derive(Debug)]
pub struct StateMachine {
    memory: Arc<Memory>,
    processor: Processor,
}

impl StateMachine {
    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.memory.resolve::<T>(program_id, resource_id)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<Resource>> {
        self.memory.insert(program_id, resource_id, resource)
    }

    pub async fn tick(&self) {
        {
            let mut next_events = self.memory.resolve::<Unique<NextEvents>>(None, None).unwrap().unwrap();
            let mut current_events = self.memory.resolve::<Unique<CurrentEvents>>(None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);

            let mut next_blockers = self.memory.resolve::<Unique<NextBlockers>>(None, None).unwrap().unwrap();
            let mut current_blockers = self.memory.resolve::<Unique<CurrentBlockers>>(None, None).unwrap().unwrap();

            current_blockers.tick(&mut next_blockers);
        }


        self.processor.tick(&self.memory).await;
    }
}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}