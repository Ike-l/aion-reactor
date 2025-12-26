use std::collections::{HashMap, HashSet};

use crate::prelude::{Criteria, EventId, ProgramKey, ProgramId, ResourceId, SchedulerOrdering, SystemId};

pub mod criteria;

#[derive(Debug)]
pub struct SystemMetadata {
    resource_id: ResourceId,
    program_id: Option<ProgramId>,
    key: Option<ProgramKey>,
    criteria: Criteria,
    ordering: SchedulerOrdering,
}

impl SystemMetadata {
    pub fn new(resource_id: ResourceId, program_id: Option<ProgramId>, key: Option<ProgramKey>, criteria: Criteria, ordering: SchedulerOrdering) -> Self {
        Self {
            resource_id,
            program_id, 
            key,
            criteria,
            ordering
        }
    }

    pub fn test(&self, events: &HashSet<&EventId>) -> bool {
        self.criteria.test(events)
    }

    pub fn program_id(&self) -> &Option<ProgramId> {
        &self.program_id
    }

    pub fn key(&self) -> &Option<ProgramKey> {
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
pub struct SystemRegistry(HashMap<SystemId, SystemMetadata>);

impl SystemRegistry {
    pub fn read(&self) -> impl Iterator<Item = (&SystemId, &SystemMetadata)> {
        self.0.iter()
    }

    pub fn into_map(&self) -> impl Iterator<Item = (SystemId, (ResourceId, Option<ProgramId>, Option<ProgramKey>))> {
        self.0.iter().map(|(id, system_metadata)| {
            let resource_id = system_metadata.resource_id().clone();
            let program_id = system_metadata.program_id().clone();
            let key = system_metadata.key().clone();
            (id.clone(), (resource_id, program_id, key))
        })
    }

    pub fn insert(&mut self, id: SystemId, system_metadata: SystemMetadata) -> Option<SystemMetadata> {
        self.0.insert(id, system_metadata)
    }

    pub fn get(&self, id: &SystemId) -> Option<&SystemMetadata> {
        self.0.get(id)
    }
}
