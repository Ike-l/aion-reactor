use std::collections::{HashMap, HashSet};

use crate::{prelude::{Criteria, EventId, Memory, SchedulerOrdering, StoredSystem, SystemCell, SystemId, Unique}, state_machine::kernel_systems::processors::system::system_metadata::stored_system_metadata::StoredSystemMetadata};

pub mod criteria;
pub mod stored_system_metadata;

#[derive(Debug)]
pub struct SystemMetadata {
    stored_system_metadata: StoredSystemMetadata,
    criteria: Criteria,
    ordering: SchedulerOrdering,
}

impl SystemMetadata {
    pub fn new(
        stored_system_metadata: StoredSystemMetadata,
        criteria: Criteria, 
        ordering: SchedulerOrdering
    ) -> Self {
        Self {
            stored_system_metadata,
            criteria,
            ordering
        }
    }

    pub fn test(&self, events: &HashSet<&EventId>) -> bool {
        self.criteria.test(events)
    }

    pub fn stored_system_metadata(&self) -> &StoredSystemMetadata {
        &self.stored_system_metadata
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

    pub fn insert(&mut self, id: SystemId, system_metadata: SystemMetadata) -> Option<SystemMetadata> {
        self.0.insert(id, system_metadata)
    }

    pub fn get(&self, id: &SystemId) -> Option<&SystemMetadata> {
        self.0.get(id)
    }

    pub fn into_system_cell_map(&self, memory: &Memory) -> HashMap<SystemId, SystemCell> {
        self.read()
            .map(|(system_id, system_metadata)| (system_id.clone(), system_metadata.stored_system_metadata().clone()))
            .filter_map(|(id, system_metadata)| {
                let stored_system = memory.resolve::<Unique<StoredSystem>>(
                    system_metadata.program_id().as_ref(), 
                    Some(system_metadata.resource_id()), 
                    None, 
                    system_metadata.key().as_ref()
                );

                match stored_system {
                    Some(Ok(mut stored_system)) => {
                        if let Some(system) = stored_system.take_system() {
                            return Some((id, SystemCell::new(system)));
                        }
                    },
                    // cant do anything (like trace) since no guarantee the system will actually be ran
                    _ => ()
                }

                None
            })
            .collect() 
    }
}
