use std::{any::type_name, sync::Arc};

use crate::{injection::{injection_trait::{Injection, MemoryTarget}, AccessDropper, DeAccessResolver}, memory::{access_map::AccessMap, errors::ResolveError, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};

#[derive(Debug, small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Global<'a, T: Injection> {
    value: T::Item<'a>,
}

impl<'a, T: Injection> Global<'a, T> {
    pub fn new(value: T::Item<'a>) -> Self {
        Self {
            value,
        }
    }
}

impl<T: Injection> AccessDropper for Global<'_, T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        self.value.access_dropper()
    }
}

impl<T: Injection> Injection for Global<'_, T> {
    type Item<'new> = Global<'new, T>;

    fn failed_message() -> String {
        format!("Expected Global Injection: `{}`. Failed with {}", type_name::<T>(), T::failed_message())
    }

    fn create_access_map() -> AccessMap {
        T::create_access_map()
    }

    fn resolve_accesses(access_map: &mut AccessMap, source: Option<&Source>, resource_id: Option<ResourceId>) {
        T::resolve_accesses(access_map, source, resource_id);
    }

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Global
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<Self::Item<'a>, ResolveError> {
        Ok(Global::new(T::retrieve(memory_domain, resource_id, source)?))
    }
}

#[cfg(test)]
mod global_tests {
    use std::{any::TypeId, sync::Arc};

    use crate::{id::Id, injection::{injection_advanced::local::Global, injection_primitives::shared::Shared}, memory::{access_checked_heap::heap::{raw_heap_object::RawHeapObject, HeapId, HeapObject}, memory_domain::MemoryDomain, resource_id::Resource, Memory, ResourceId}};

    #[test]
    fn resolve_global_fails_no_res() {
        let memory_domain = Arc::new(MemoryDomain::new());
        assert!(memory_domain.resolve::<Global<Shared<i32>>>(None, None).is_err())
    }

    #[test]
    fn resolve_global_shared() {
        let memory_domain = Arc::new(MemoryDomain::new());
        
        assert!(memory_domain.insert(
            ResourceId::Heap(
                HeapId::RawType(
                    TypeId::of::<i32>()
                )
            ), 
            Resource::Heap(HeapObject(RawHeapObject::new(Box::new(1 as i32))))
        ).unwrap().is_none());

        assert_eq!(***memory_domain.resolve::<Global<Shared<i32>>>(None, None).unwrap(), 1 as i32);
    }

    #[test]
    fn resolve_global_memory() {
        let program_id = Id("Foo".to_string());
        let memory_domain = Arc::new(MemoryDomain::new());
        
        assert!(memory_domain.insert(ResourceId::Heap(HeapId::RawType(TypeId::of::<i32>())), Resource::dummy(1)).unwrap().is_none());

        let memory = Memory::new();
        assert!(memory.insert_program(program_id.clone(), memory_domain, None));

        memory.insert(None, None, None, 2 as i32);

        assert_eq!(***memory.resolve::<Global<Shared<i32>>>(Some(&program_id), None, None, None).unwrap().unwrap(), 2 as i32);
        assert_eq!(**memory.resolve::<Shared<i32>>(Some(&program_id), None, None, None).unwrap().unwrap(), 1 as i32);
        assert_eq!(**memory.resolve::<Shared<i32>>(None, None, None, None).unwrap().unwrap(), 2 as i32);
        assert_eq!(***memory.resolve::<Global<Shared<i32>>>(None, None, None, None).unwrap().unwrap(), 2 as i32);
    }
}