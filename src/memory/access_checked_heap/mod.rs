use std::sync::Mutex;

use crate::prelude::{Access, DeResolveError, Heap, HeapId, HeapObject, InsertError, MemoryDomain, RawAccessMap, ReservationAccessMap, ReservationError, ResolveError, ResourceId, SystemId};

pub mod heap;
pub mod reservation_access_map;
pub mod reserve_access_map;
pub mod raw_access_map;


#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    reservation_access_map: Mutex<ReservationAccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn ok_resource(&self, heap_id: &HeapId) -> bool {
        self.heap.contains(heap_id)
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access, system_id: Option<&SystemId>) -> bool {
        let access_map = self.reservation_access_map.lock().unwrap();
        self.ok_resource(testing_heap_id) && access_map.ok_access(testing_heap_id, testing_access, system_id)
    }

    pub fn unreserve(&self, heap_id: &HeapId, access: Access, system_id: &SystemId) {
        self.reservation_access_map.lock().unwrap().unreserve(heap_id, access, system_id)
    }

    /// Will drain the access map
    pub fn reserve_accesses(&self, memory_domain: &MemoryDomain, system_id: SystemId, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        self.reservation_access_map.lock().unwrap().reserve_accesses(memory_domain, system_id, access_map)
    }

    /// Will drain the access map
    pub fn reserve_accesses_self(&self, memory_domain: &MemoryDomain, system_id: SystemId, other: &mut Self) -> Result<(), ReservationError> {
        self.reservation_access_map.lock().unwrap().reserve_accesses_self(memory_domain, system_id, &mut other.reservation_access_map.lock().unwrap())
    }

    pub fn reserve_current_accesses(&self, system_id: SystemId, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        self.reservation_access_map.lock().unwrap().reserve_current_accesses(system_id, access_map)
    }

    pub fn ok_reservation_self(&self, other: &Self, system_id: Option<&SystemId>, memory_domain: &MemoryDomain) -> Option<ReservationError> {
        self.reservation_access_map.lock().unwrap().ok_reservation_self(&mut other.reservation_access_map.lock().unwrap(), system_id, memory_domain)
    }

    pub fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Result<Option<HeapObject>, InsertError> {
        let access_map = self.reservation_access_map.lock().unwrap();
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
        self.reservation_access_map.lock().unwrap().deaccess(access, heap_id)
    }

    pub fn get_cloned<T: 'static + Clone>(&self, heap_id: &HeapId) -> Result<T, ResolveError> {
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok(self.heap.get::<T>(&heap_id).ok_or(ResolveError::NoResource(ResourceId::Heap(heap_id.clone())))?.clone())
        }
    }

    pub fn get_shared<T: 'static>(&self, heap_id: &HeapId, system_id: Option<&SystemId>) -> Result<&T, ResolveError> {
        let mut access_map = self.reservation_access_map.lock().unwrap();

        // Safety:
        // Accesses are tracked
        if let Some(result) = unsafe { self.heap.get::<T>(&heap_id) } {
            access_map.do_access(heap_id.clone(), system_id, Access::Shared(1))?;

            Ok(result)
        } else {
            Err(ResolveError::NoResource(ResourceId::Heap(heap_id.clone())))
        }
    }

    pub fn get_unique<T: 'static>(&self, heap_id: &HeapId, system_id: Option<&SystemId>) -> Result<&mut T, ResolveError> {
        let mut access_map = self.reservation_access_map.lock().unwrap();

        // Safety:
        // Accesses are tracked
        if let Some(result) = unsafe { self.heap.get_mut::<T>(&heap_id) } {
            access_map.do_access(heap_id.clone(), system_id, Access::Unique)?;

            Ok(result)
        } else {
            Err(ResolveError::NoResource(ResourceId::Heap(heap_id.clone())))
        }
    }
}

#[cfg(test)]
mod access_checked_heap_tests {
    use crate::prelude::{Access, AccessCheckedHeap, HeapId, HeapObject, Id, MemoryDomain, RawAccessMap, Resource, ResourceId, SystemId};

    #[test]
    fn insert() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id::from("foo"));

        let testing_access = Access::Unique;
        let system_id = None;

        assert!(!access_checked_heap.ok_resource(&heap_id));
        assert!(!access_checked_heap.ok_access(&heap_id, &testing_access, system_id));

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(100)).unwrap().is_none());
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(101)).unwrap().is_some());

        assert!(access_checked_heap.ok_resource(&heap_id));
        assert!(access_checked_heap.ok_access(&heap_id, &testing_access, system_id));


        let r = access_checked_heap.get_shared::<i32>(&heap_id, None);
        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(102)).is_err());
        assert_eq!(r, Ok(&101))
    }

    #[test]
    fn get_shared() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id::from("foo"));

        let system_id = None;

        assert!(access_checked_heap.get_shared::<i32>(&heap_id, system_id).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
        let first = access_checked_heap.get_shared::<i32>(&heap_id, system_id);
        assert_eq!(first, Ok(&1));

        let second = access_checked_heap.get_shared::<i32>(&heap_id, system_id);
        assert_eq!(second, Ok(&1));
    }

    #[test]
    fn get_unique() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id::from("foo"));

        let system_id = None;

        assert!(access_checked_heap.get_unique::<i32>(&heap_id, system_id).is_err());

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
        let first = access_checked_heap.get_unique::<i32>(&heap_id, system_id);
        assert_eq!(first, Ok(&mut 1));

        let second = access_checked_heap.get_unique::<i32>(&heap_id, system_id);
        assert!(second.is_err());
    }

    #[test]
    fn get_shared_and_unique() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id::from("foo"));

        let system_id = None;

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(1)).unwrap().is_none());
        
        let first = access_checked_heap.get_shared::<i32>(&heap_id, system_id);
        assert_eq!(first, Ok(&1));

        let second = access_checked_heap.get_unique::<i32>(&heap_id, system_id);
        assert!(second.is_err());

        let third = access_checked_heap.get_shared::<i32>(&heap_id, system_id);
        assert_eq!(third, Ok(&1));

        assert!(unsafe { access_checked_heap.deaccess(Access::Shared(2), &heap_id).is_ok() } );

        let second = access_checked_heap.get_unique::<i32>(&heap_id, system_id);
        assert_eq!(second, Ok(&mut 1));

        assert!(access_checked_heap.get_shared::<i32>(&heap_id, system_id).is_err());

        assert_eq!(access_checked_heap.get_cloned(&heap_id), Ok(1));
    }

    #[test]
    fn get_cloned() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id = HeapId::Label(Id::from("foo"));

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
        let heap_id = HeapId::Label(Id::from("foo"));

        let system_id = None;

        access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(123)).unwrap();

        access_checked_heap.get_shared::<i32>(&heap_id, system_id).unwrap();
        assert!(access_checked_heap.get_unique::<i32>(&heap_id, system_id).is_err());
        unsafe { access_checked_heap.deaccess(Access::Shared(1), &heap_id).unwrap() };

        assert_eq!(access_checked_heap.get_unique::<i32>(&heap_id, system_id), Ok(&mut 123));
    }

    // reserve - see MemoryDomain (unable to test from just here due to visitor pattern)

    #[test]
    fn ok_access() {
        let access_checked_heap = AccessCheckedHeap::default();

        let heap_id = HeapId::Label(Id::from("foo"));
        let access = Access::Unique;
        let system_id = None;

        assert!(!access_checked_heap.ok_access(&heap_id, &access, system_id));

        assert!(access_checked_heap.insert(heap_id.clone(), HeapObject::dummy(123)).is_ok());
        assert!(access_checked_heap.ok_access(&heap_id, &access, system_id));
        assert!(access_checked_heap.get_shared::<i32>(&heap_id, system_id).is_ok());
        assert!(access_checked_heap.get_shared::<i32>(&heap_id, system_id).is_ok());

        assert!(!access_checked_heap.ok_access(&heap_id, &access, system_id));

        assert!(unsafe { access_checked_heap.deaccess(Access::Shared(2), &heap_id) }.is_ok());
        assert!(access_checked_heap.ok_access(&heap_id, &access, system_id));
        assert!(access_checked_heap.get_unique::<i32>(&heap_id, system_id).is_ok());
        assert!(unsafe { access_checked_heap.deaccess(Access::Unique, &heap_id) }.is_ok());

        // warning: not how this function is intended
        let access_map = &mut RawAccessMap::default();
        assert!(access_map.do_access(heap_id.clone(), access.clone()).is_ok());

        let memory_domain = MemoryDomain::new();
        assert!(memory_domain.insert(ResourceId::Heap(heap_id.clone()), Resource::dummy(123)).is_ok());

        let system_id = SystemId::from("foo");
        assert_eq!(access_checked_heap.reserve_accesses(&memory_domain, system_id.clone(), access_map), Ok(()));


        assert!(access_checked_heap.ok_access(&heap_id, &access, Some(&system_id)));
        assert!(!access_checked_heap.ok_access(&heap_id, &access, None));

        access_checked_heap.unreserve(&heap_id, access.clone(), &system_id);
        assert!(access_checked_heap.ok_access(&heap_id, &access, None));
    }

    #[test]
    fn ok_resource() {
        let access_checked_heap = AccessCheckedHeap::default();
        let heap_id1 = HeapId::Label(Id::from("foo"));
        let heap_id2 = HeapId::Label(Id::from("bar"));

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