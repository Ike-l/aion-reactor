use std::{collections::HashMap, sync::Arc};

use crate::{id::Id, memory::memory_domain::MemoryDomain};

pub type Key = u64;

#[derive(Debug, Default)]
pub struct InnerProgramMemoryMap {
    memory_map: HashMap<Id, Arc<MemoryDomain>>,
    key_map: HashMap<Id, Key>
}

impl InnerProgramMemoryMap {
    pub fn insert(&mut self, program_id: Id, memory_domain: Arc<MemoryDomain>, key: Option<Key>) -> Option<Arc<MemoryDomain>> {
        if let Some(key) = key {
            self.key_map.insert(program_id.clone(), key);
        }

        self.memory_map.insert(program_id, memory_domain)
    }

    pub fn contains(&self, program_id: &Id) -> bool {
        self.memory_map.contains_key(program_id)
    }

    pub fn get(&self, program_id: &Id, key: Option<&Key>) -> Option<&Arc<MemoryDomain>> {
        if let Some(registered_key) = self.key_map.get(program_id) {
            if registered_key != key? {
                return None
            }
        }
        
        self.memory_map.get(program_id)
    }

    pub fn consume(mut self) -> impl Iterator<Item = (Option<Key>, Id, Arc<MemoryDomain>)> {
        self.memory_map.into_iter().map(move |(id, memory_domain)| (self.key_map.remove(&id), id, memory_domain, ))
    }
}