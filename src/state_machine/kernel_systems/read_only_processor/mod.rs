use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::{injection_primitives::{cloned::Cloned, shared::Shared}, injection_trait::Injection}, kernel_prelude::KernelSystem, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}};

pub struct ReadOnlyProcessorManager;

impl KernelSystem for ReadOnlyProcessorManager {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        // "system registry"
        // mapping of system id: Vec<Event>
        ResourceId::Heap(HeapId::Label(Id("KernelReadOnlyProcessorManager".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            // ReadOnlySystem
            // implemented for systems with ReadOnlyInjection
            // has function `check_read_only`: for each injection, Injection::CreateAccessMap.is_read_only()
            // when registering system call this function, if false -> reject
            // else insert system into registry
            
            // collect systems: Processor::get_systems(memory, read_only_registry)
            // Execute:
            // for each thread in threadpool:
            // thread.spawn(|| {
            // for each system in chunk:
            //  run system
            //  if matches(run(system), SystemResult::Conditional(true))
            //      spawn next event associated_event(system)
            // })
            
            // CreateAccessMap:
            // let mut access_map = Self::create_access_map();
            // Self::resolve_accesses(&mut access_map, source, Some(accessing.clone()));
        })
    }
}

pub trait ReadOnlyInjection: Injection {}

impl<T: 'static> ReadOnlyInjection for Shared<'_, T> {}
impl<T: Clone + 'static> ReadOnlyInjection for Cloned<T> {}
