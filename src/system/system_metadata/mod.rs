use std::collections::{HashMap, HashSet};

use crate::{id::Id, memory::ResourceId, state_machine::kernel_systems::{event_manager::event::Event, processor::scheduler::ordering::SchedulerOrdering}, system::system_metadata::criteria::Criteria};


pub mod criteria;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Source(pub ResourceId);

#[derive(Debug)]
pub struct SystemMetadata {
    resource_id: Source,
    program_id: Option<Id>,
    criteria: Criteria,
    ordering: SchedulerOrdering,
}

impl SystemMetadata {
    pub fn test(&self, events: &HashSet<&Event>) -> bool {
        self.criteria.test(events)
    }

    pub fn ids(&self) -> (&Source, &Option<Id>) {
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

    pub fn into_map(&self) -> impl Iterator<Item = (Id, (Source, Option<Id>))> {
        self.0.iter().map(|(id, system_metadata)| {
            let (source, program_id) = system_metadata.ids();
            (id.clone(), (source.clone(), program_id.clone()))
        })
    }
}
