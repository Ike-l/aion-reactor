use std::{collections::HashMap, sync::Arc};

use crate::{id::Id, memory::memory_domain::MemoryDomain};

#[derive(Debug, Default)]
pub struct InnerProgramMemoryMap(HashMap<Id, Arc<MemoryDomain>>);

impl InnerProgramMemoryMap {
    pub fn insert(&mut self, program_id: Id, memory_domain: Arc<MemoryDomain>) -> Option<Arc<MemoryDomain>> {
        self.0.insert(program_id, memory_domain)
    }

    pub fn contains(&self, program_id: &Id) -> bool {
        self.0.contains_key(program_id)
    }

    pub fn get(&self, program_id: &Id) -> Option<&Arc<MemoryDomain>> {
        self.0.get(program_id)
    }
}