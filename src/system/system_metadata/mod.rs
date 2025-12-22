use std::collections::{HashMap, HashSet};

use crate::{id::Id, memory::{program_memory_map::inner_program_memory_map::Key, ResourceId}, state_machine::kernel_systems::{event_manager::event::Event, processor::scheduler::ordering::SchedulerOrdering}, system::system_metadata::criteria::Criteria};


pub mod criteria;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Source(pub Id);

#[derive(Debug)]
pub struct SystemMetadata {
    resource_id: ResourceId,
    program_id: Option<Id>,
    key: Option<Key>,
    criteria: Criteria,
    ordering: SchedulerOrdering,
}

impl SystemMetadata {
    pub fn new(resource_id: ResourceId, program_id: Option<Id>, key: Option<Key>, criteria: Criteria, ordering: SchedulerOrdering) -> Self {
        Self {
            resource_id,
            program_id, 
            key,
            criteria,
            ordering
        }
    }

    pub fn test(&self, events: &HashSet<&Event>) -> bool {
        self.criteria.test(events)
    }

    pub fn program_id(&self) -> &Option<Id> {
        &self.program_id
    }

    pub fn key(&self) -> &Option<Key> {
        &self.key
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn ordering(&self) -> &SchedulerOrdering {
        &self.ordering
    }

    pub fn insert_ordering(&mut self, ordering: SchedulerOrdering) {
        self.ordering.consume(ordering);
    }

    pub fn replace_criteria(&mut self, criteria: Criteria) {
        self.criteria = criteria;
    }
}

#[derive(Debug, Default)]
pub struct SystemRegistry(HashMap<Id, SystemMetadata>);

impl SystemRegistry {
    pub fn read(&self) -> impl Iterator<Item = (&Id, &SystemMetadata)> {
        self.0.iter()
    }

    pub fn into_map(&self) -> impl Iterator<Item = (Id, (ResourceId, Option<Id>, Option<Key>))> {
        self.0.iter().map(|(id, system_metadata)| {
            let resource_id = system_metadata.resource_id().clone();
            let program_id = system_metadata.program_id().clone();
            let key = system_metadata.key().clone();
            (id.clone(), (resource_id, program_id, key))
        })
    }

    pub fn insert(&mut self, id: Id, system_metadata: SystemMetadata) -> Option<SystemMetadata> {
        self.0.insert(id, system_metadata)
    }

    pub fn get(&self, id: &Id) -> Option<&SystemMetadata> {
        self.0.get(id)
    }
}
