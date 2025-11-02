use std::sync::Mutex;

use crate::{memory::{ResourceId, access_checked_heap::{heap::{HeapId, HeapObject, heap::Heap}, raw_access_map::RawAccessMap, reservation_access_map::ReservationAccessMap}, access_map::Access, errors::{DeResolveError, InsertError, ReservationError, ResolveError}, memory_domain::MemoryDomain}, system::system_metadata::Source};

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

    pub fn unreserve(&self, heap_id: &HeapId, access: Access, source: &Source) {
        self.access_map.lock().unwrap().unreserve(heap_id, access, source)
    }

    /// Will drain the access map
    pub fn reserve_accesses(&self, memory_domain: &MemoryDomain, source: Source, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        self.access_map.lock().unwrap().reserve_accesses(memory_domain, source, access_map)
    }

    pub fn reserve_current_accesses(&self, source: Source, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        self.access_map.lock().unwrap().reserve_current_accesses(source, access_map)
    }

    pub fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Result<Option<HeapObject>, InsertError> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.get_access(&heap_id) {
            return Err(InsertError::ConcurrentAccess)
        }

        // Safety:
        // Accesses are tracked
        // No Access allowed
        Ok(unsafe { self.heap.insert(heap_id, resource) })
    }

    // pub crate for now since i only want the dropper to use this
    /// Safety:
    /// Do not deaccess something unless you actually free the access!
    pub(crate) unsafe fn deaccess(&self, access: Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
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
    use crate::{id::Id, memory::{ResourceId, access_checked_heap::{AccessCheckedHeap, heap::{HeapId, HeapObject}, raw_access_map::RawAccessMap}, access_map::Access, memory_domain::MemoryDomain, resource_id::Resource}, system::system_metadata::Source};

    #[test]
    fn insert() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let testing_access = Access::Unique;
        let source = None;

        assert!(!access_checked_heap.ok_resource(&heap_id));
        assert!(!access_checked_heap.ok_access(&heap_id, &testing_access, source));

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(100)).unwrap().is_none());
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(101)).unwrap().is_some());

        assert!(access_checked_heap.ok_resource(&heap_id));
        assert!(access_checked_heap.ok_access(&heap_id, &testing_access, source));


        let r = access_checked_heap.get_shared::<i32>(&heap_id, None);
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(102)).is_err());
        assert_eq!(r, Ok(&101))
    }

    #[test]
    fn get_shared() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let source = None;

        assert!(access_checked_heap.get_shared::<i32>(&heap_id, source).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
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

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
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

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
        let first = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(first, Ok(&1));

        let second = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert!(second.is_err());

        let third = access_checked_heap.get_shared::<i32>(&heap_id, source);
        assert_eq!(third, Ok(&1));

        assert!(unsafe { access_checked_heap.deaccess(Access::Shared(2), &heap_id).is_ok() } );

        let second = access_checked_heap.get_unique::<i32>(&heap_id, source);
        assert_eq!(second, Ok(&mut 1));

        assert!(access_checked_heap.get_shared::<i32>(&heap_id, source).is_err());

        assert_eq!(access_checked_heap.get_cloned(&heap_id), Ok(1));
    }

    #[test]
    fn get_cloned() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(access_checked_heap.get_cloned::<i32>(&heap_id).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());

        assert_eq!(access_checked_heap.get_cloned::<i32>(&heap_id), Ok(1));
        assert_eq!(access_checked_heap.get_cloned::<i32>(&heap_id), Ok(1));
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(2)).unwrap().is_some());
        assert_eq!(access_checked_heap.get_cloned::<i32>(&heap_id), Ok(2));
    }

    #[test]
    fn deaccess() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id("foo".to_string()));

        let source = None;

        access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(123)).unwrap();

        access_checked_heap.get_shared::<i32>(&heap_id, source).unwrap();
        assert!(access_checked_heap.get_unique::<i32>(&heap_id, source).is_err());
        unsafe { access_checked_heap.deaccess(Access::Shared(1), &heap_id).unwrap() };

        assert_eq!(access_checked_heap.get_unique::<i32>(&heap_id, source), Ok(&mut 123));
    }

    // reserve - see MemoryDomain (unable to test from just here due to visitor pattern)

    #[test]
    fn ok_access() {
        let access_checked_heap = AccessCheckedHeap::default();

        let heap_id = HeapId::Label(Id("foo".to_string()));
        let access = Access::Unique;
        let source = None;

        assert!(!access_checked_heap.ok_access(&heap_id, &access, source));

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(123)).is_ok());
        assert!(access_checked_heap.ok_access(&heap_id, &access, source));
        assert!(access_checked_heap.get_shared::<i32>(&heap_id, source).is_ok());
        assert!(access_checked_heap.get_shared::<i32>(&heap_id, source).is_ok());

        assert!(!access_checked_heap.ok_access(&heap_id, &access, source));

        assert!(unsafe { access_checked_heap.deaccess(Access::Shared(2), &heap_id) }.is_ok());
        assert!(access_checked_heap.ok_access(&heap_id, &access, source));
        assert!(access_checked_heap.get_unique::<i32>(&heap_id, source).is_ok());
        assert!(unsafe { access_checked_heap.deaccess(Access::Unique, &heap_id) }.is_ok());

        // warning: not how this function is intended
        let access_map = &mut RawAccessMap::default();
        assert!(access_map.do_access(heap_id.clone(), access.clone()).is_ok());

        let memory_domain = MemoryDomain::new();
        assert!(memory_domain.insert(ResourceId::Heap(heap_id.clone()), Resource::dummy(123)).is_ok());

        let source = Source(Id("foo".to_string()));
        assert_eq!(access_checked_heap.reserve_accesses(&memory_domain, source.clone(), access_map), Ok(()));


        assert!(access_checked_heap.ok_access(&heap_id, &access, Some(&source)));
        assert!(!access_checked_heap.ok_access(&heap_id, &access, None));

        access_checked_heap.unreserve(&heap_id, access.clone(), &source);
        assert!(access_checked_heap.ok_access(&heap_id, &access, None));
    }

    #[test]
    fn ok_resource() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id1 = HeapId::Label(Id("foo".to_string()));
        let heap_id2 = HeapId::Label(Id("bar".to_string()));

        assert!(!access_checked_heap.ok_resource(&heap_id1));
        assert!(!access_checked_heap.ok_resource(&heap_id2));

        assert!(access_checked_heap.insert(heap_id1.clone(), HeapObject::dummy(123)).is_ok());
        
        assert!(access_checked_heap.ok_resource(&heap_id1));
        assert!(!access_checked_heap.ok_resource(&heap_id2));
        
        assert!(access_checked_heap.insert(heap_id2.clone(), HeapObject::dummy(123)).is_ok());
        
        assert!(access_checked_heap.ok_resource(&heap_id1));
        assert!(access_checked_heap.ok_resource(&heap_id2));
    }
}