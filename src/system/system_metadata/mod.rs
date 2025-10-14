use std::collections::{HashMap, HashSet};

use crate::{id::Id, memory::ResourceId, state_machine::kernel_systems::processor::scheduler::ordering::SchedulerOrdering, state_machine::event::Event, system::system_metadata::{criteria::Criteria, system_kind::SystemKind}};


pub mod system_kind;
pub mod criteria;

#[derive(Debug)]
pub struct SystemMetadata {
    system_kind: SystemKind,
    resource_id: ResourceId,
    program_id: Option<Id>,
    criteria: Criteria,
    ordering: SchedulerOrdering,
}

impl SystemMetadata {
    pub fn test(&self, events: &HashSet<&Event>) -> bool {
        self.criteria.test(events)
    }

    pub fn ids(&self) -> (&ResourceId, &Option<Id>) {
        (&self.resource_id, &self.program_id)
    }

    pub fn ordering(&self) -> &SchedulerOrdering {
        &self.ordering
    }
}

#[derive(Debug)]
pub struct SystemRegistry(HashMap<Id, SystemMetadata>);

impl SystemRegistry {
    pub fn read(&self) -> impl Iterator<Item = (&Id, &SystemMetadata)> {
        self.0.iter()
    }

    pub fn into_map(&self) -> impl Iterator<Item = (Id, (ResourceId, Option<Id>))> {
        self.0.iter().map(|(id, system_metadata)| {
            let (resource_id, program_id) = system_metadata.ids();
            (id.clone(), (resource_id.clone(), program_id.clone()))
        })
    }
}
