pub mod injection_trait;
pub mod injection_primitives;

use std::sync::Arc;

use crate::{memory::{access_map::AccessMap, memory_domain::MemoryDomain, }};

pub trait AccessDropper {
    fn access_dropper(&self) -> &AccessDeResolver; 
}

pub struct AccessDeResolver {
    memory_domain: Arc<MemoryDomain>,
    access_map: AccessMap
}

macro_rules! resolve {
    ($memory:ident, $program_id:ident, $resource_id:ident) => { { Ok($memory.resolve::<Self>($program_id, $resource_id).unwrap_or_else(|| Err(ResolveError::InvalidProgramId))) } };
}

pub(crate) use resolve;

impl AccessDeResolver {
    fn new(memory_domain: Arc<MemoryDomain>, access_map: AccessMap) -> Self {
        Self { memory_domain: memory_domain, access_map }
    }
}

impl Drop for AccessDeResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.drain() {
            self.memory_domain.deresolve(access, &resource).unwrap();
        }
    }
}