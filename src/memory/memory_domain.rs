use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{injection::{injection_trait::Injection, AccessDropper}, memory::{access_checked_heap::AccessCheckedHeap, access_map::{Access, AccessMap}, errors::{DeResolveError, ResolveError}, resource_id::Resource, ResourceId}, system::system_metadata::Source};

// Should be no public way of creating one of these to enforce dropping behaviour by injection types // doesnt matter because the UB would just panic
#[derive(Debug)]
pub struct MemoryDomain {
    heap: AccessCheckedHeap,

    delays: Mutex<HashMap<u64, HashMap<ResourceId, Access>>>
}

impl MemoryDomain {
    pub fn new() -> Self {
        Self {
            heap: AccessCheckedHeap::default(),
            delays: Mutex::new(HashMap::new())
        }
    }

    pub fn ok_resource(&self, resource_id: &ResourceId) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_resource(&heap_id)
        }
    }

    pub fn ok_access(&self, resource_id: &ResourceId, access: &Access) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_access(&heap_id, access)
        }
    }

    pub fn reserve_accesses(&self, source: Source, access_map: AccessMap) -> bool {
        match access_map {
            AccessMap::Heap(access_map) => self.heap.reserve_accesses(&self, source, access_map)
        }
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Resource) -> Option<Resource> {
        match (resource, resource_id) {
            (Resource::Heap(obj), ResourceId::Heap(id)) => Some(Resource::Heap(self.heap.insert(id, obj)?))
        }
    }

    pub fn resolve<T: Injection>(self: &Arc<Self>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self, resource_id, source);
        if let Ok(r) = &r {
            // make sure no panics so there MUST be a dropper
            std::hint::black_box(r.access_dropper());
        }

        r
    }

    pub(crate) fn delay_drop(&self, accesses: HashMap<ResourceId, Access>) -> u64 {
        let key = rand::random();
        self.delays.lock().unwrap().insert(key, accesses);
        key
    }

    pub fn end_drop_delay(&self, key: &u64) {
        if let Some(accesses) = self.delays.lock().unwrap().remove(key) {
            for (resource_id, access) in accesses {
                self.deresolve(&access, &resource_id).unwrap();
            }
        } else {
            panic!("tried to end the drop delay without permission")
        }
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: &Access, resource_id: &ResourceId) -> Result<(), DeResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.deresolve(access, id)
        }
    }

    pub fn get_cloned<T: 'static + Clone>(&self, resource_id: &ResourceId) -> Result<T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_cloned(id)
        }
    }

    pub fn get_shared<T: 'static>(&self, resource_id: &ResourceId, source: Option<&Source>) -> Result<&T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_shared(id, source)
        }
    }

    pub fn get_unique<T: 'static>(&self, resource_id: &ResourceId, source: Option<&Source>) -> Result<&mut T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_unique(id, source)
        }
    }
}