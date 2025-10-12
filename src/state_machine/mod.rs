use crate::{id::Id, injection::injection_trait::Injection, memory::{access_checked_resource_map::{heap::HeapObject, ResolveError}, Memory, ResourceId}, processor::Processor, system::stored_system::StoredSystem};

pub mod event;

#[derive(Debug)]
pub struct StateMachine {
    memory: Memory,
    processor: Processor,
    system_metadata: Vec<StoredSystem>,
}

impl StateMachine {
    pub fn resolve<T: Injection>(&mut self, program_id: Option<Id>, resource_id: Option<ResourceId>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.memory.resolve::<T>(program_id, resource_id)
    }

    pub fn insert<T: 'static>(&mut self, program_id: Option<Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<HeapObject>> {
        self.memory.insert(program_id, resource_id, resource)
    }

    pub async fn tick(&mut self) {
        self.processor.tick().await;
    }

}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}