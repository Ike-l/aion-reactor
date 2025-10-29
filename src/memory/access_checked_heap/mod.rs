use std::sync::Mutex;

use crate::{memory::{ResourceId, access_checked_heap::{heap::{HeapId, HeapObject, heap::Heap}, raw_access_map::RawAccessMap, reservation_access_map::ReservationAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain}, system::system_metadata::Source};

pub mod heap;
pub mod reservation_access_map;
pub mod reserve_access_map;
pub mod raw_access_map;


#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    access_map: Mutex<ReservationAccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn ok_resource(&self, heap_id: &HeapId) -> bool {
        self.heap.contains(heap_id)
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access, source: Option<&Source>) -> bool {
        let access_map = self.access_map.lock().unwrap();
        self.ok_resource(testing_heap_id) && access_map.ok_access(testing_heap_id, testing_access, source)
    }

    pub fn reserve_accesses(&self, memory_domain: &MemoryDomain, source: Source, access_map: &mut RawAccessMap) -> bool {
        self.access_map.lock().unwrap().reserve_accesses(memory_domain, source, access_map)
    }

    pub fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.get_access(&heap_id) {
            return None
        }

        // Safety:
        // Accesses are tracked
        // No Access allowed
        unsafe { self.heap.insert(heap_id, resource) }
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deaccess(&self, access: Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        self.access_map.lock().unwrap().deaccess(access, heap_id)
    }

    pub fn get_cloned<T: 'static + Clone>(&self, heap_id: &HeapId) -> Result<T, ResolveError> {
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok(self.heap.get::<T>(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?.clone())
        }
    }

    pub fn get_shared<T: 'static>(&self, heap_id: &HeapId, source: Option<&Source>) -> Result<&T, ResolveError> {
        let mut access_map = self.access_map.lock().unwrap();

        // Safety:
        // Accesses are tracked
        if let Some(result) = unsafe { self.heap.get::<T>(&heap_id) } {
            access_map.do_access(heap_id.clone(), source, Access::Shared(1))?;

            Ok(result)
        } else {
            Err(ResolveError::NoResource(ResourceId::from(heap_id.clone())))
        }
    }

    pub fn get_unique<T: 'static>(&self, heap_id: &HeapId, source: Option<&Source>) -> Result<&mut T, ResolveError> {
        let mut access_map = self.access_map.lock().unwrap();

        // Safety:
        // Accesses are tracked
        if let Some(result) = unsafe { self.heap.get_mut::<T>(&heap_id) } {
            access_map.do_access(heap_id.clone(), source, Access::Unique)?;

            Ok(result)
        } else {
            Err(ResolveError::NoResource(ResourceId::from(heap_id.clone())))
        }
    }
}

#[cfg(test)]
mod access_checked_heap_tests {
    use crate::{id::Id, memory::{access_checked_heap::{AccessCheckedHeap, heap::{HeapId, HeapObject}}, access_map::Access}};

    #[test]
    fn insert() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let testing_access = Access::Unique;
        let source = None;

        assert!(!access_checked_heap.ok_resource(&heap_id));
        assert!(!access_checked_heap.ok_access(&heap_id, &testing_access, source));

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(100)).is_none());
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(100)).is_some());

        assert!(access_checked_heap.ok_resource(&heap_id));
        assert!(access_checked_heap.ok_access(&heap_id, &testing_access, source));
    }

    #[test]
    fn get_shared() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let source = None;

        assert!(access_checked_heap.get_shared::<i32>(&heap_id, source).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).is_none());
        
        let first = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(first, Ok(&1));

        let second = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(second, Ok(&1));
    }

    #[test]
    fn get_unique() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let source = None;

        assert!(access_checked_heap.get_unique::<i32>(&heap_id, source).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).is_none());
        
        let first = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert_eq!(first, Ok(&mut 1));

        let second = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert!(second.is_err());
    }

    #[test]
    fn get_shared_and_unique() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let source = None;

        assert!(access_checked_heap.get_unique::<i32>(&heap_id, source).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).is_none());
        
        let first = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(first, Ok(&1));

        let second = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert!(second.is_err());

        let third = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(third, Ok(&1));

        assert!(access_checked_heap.deaccess(Access::Shared(2), &heap_id).is_ok());

        let second = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert_eq!(second, Ok(&mut 1));
    }

    #[test]
    fn get_cloned() {
        todo!()
    }

    #[test]
    fn deaccess() {
        todo!()
    }

    #[test]
    fn reserve_access() {
        todo!()
    }

    #[test]
    fn ok_access() {
        todo!()
    }

    #[test]
    fn ok_resource() {
        todo!()
    }
}