pub mod injection_trait;
pub mod injection_primitives;
pub mod injection_advanced;

use std::sync::{Arc, Mutex};

use crate::{memory::{access_map::AccessMap, memory_domain::MemoryDomain, }};

pub trait AccessDropper {
    fn access_dropper(&self) -> &DeAccessResolver; 
}

#[derive(Debug)]
pub struct DeAccessResolver {
    memory_domain: Arc<MemoryDomain>,
    access_map: Mutex<AccessMap>
}

impl DeAccessResolver {
    pub fn new(memory_domain: Arc<MemoryDomain>, access_map: AccessMap) -> Self {
        Self { memory_domain, access_map: Mutex::new(access_map) }
    }

    pub fn delay_dropper(&self) -> u64 {
        self.memory_domain.delay_drop(self.access_map.lock().unwrap().drain().collect())
    }
}

impl Drop for DeAccessResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.lock().unwrap().drain() {
            println!("Resource: {resource:?}. Access: {access:?}");
            self.memory_domain.deresolve(&access, &resource).unwrap();
        }
    }
}