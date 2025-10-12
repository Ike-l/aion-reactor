use std::{any::TypeId, collections::HashMap};

use crate::{id::Id, injection::injection_trait::Injection, memory::{access_checked_resource_map::{resource::Resource, ResolveError}, Memory}, system::stored_system::StoredSystem};


#[derive(Debug)]
pub struct StateMachine {
    memory: Memory,
    program_pointers: HashMap<Id, StoredSystem>,
}

impl StateMachine {
    pub fn resolve<T: Injection>(&mut self, program_id: Option<Id>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.memory.resolve::<T>(program_id)
    }

    pub fn insert<T: 'static>(&mut self, program_id: Option<Id>, resource: T) -> Option<Option<Resource>> {
        self.memory.insert(program_id, resource)
    }

    pub async fn tick(&mut self) {

    }

}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}