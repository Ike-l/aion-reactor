pub mod injection_trait;
pub mod injection_primitives;

use std::sync::Arc;

use crate::{memory::{access_map::AccessMap, memory_domain::MemoryDomain, }};

pub trait AccessDropper {
    fn access_dropper(&self) -> &AccessDeResolver; 
}

#[derive(Debug)]
pub struct AccessDeResolver {
    memory_domain: Arc<MemoryDomain>,
    access_map: AccessMap
}

impl AccessDeResolver {
    fn new(memory_domain: Arc<MemoryDomain>, access_map: AccessMap) -> Self {
        Self { memory_domain: memory_domain, access_map }
    }
}

impl Drop for AccessDeResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.drain() {
            println!("Resource: {resource:?}. Access: {access:?}");
            self.memory_domain.deresolve(&access, &resource).unwrap();
        }
    }
}