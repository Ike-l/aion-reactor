use std::{any::type_name, sync::Arc};

use crate::prelude::{AccessDropper, AccessMap, DeAccessResolver, Injection, MemoryDomain, MemoryTarget, ResolveError, ResourceId, SystemId};

#[derive(small_derive_deref::Deref, small_derive_deref::DerefMut)]
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

    fn resolve_accesses(access_map: &mut AccessMap, system_id: Option<&SystemId>, resource_id: Option<ResourceId>) {
        T::resolve_accesses(access_map, system_id, resource_id);
    }

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Global
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>) -> Result<Self::Item<'a>, ResolveError> {
        Ok(Global::new(T::retrieve(memory_domain, resource_id, system_id)?))
    }
}

#[cfg(test)]
mod global_tests {
    use std::sync::Arc;

    use crate::{memory::Memory, prelude::{ProgramId, Resource, Shared}};

    use super::*;
    
    #[test]
    fn resolve_global_fails_no_res() {
        let memory_domain = Arc::new(MemoryDomain::new());
        assert!(memory_domain.resolve::<Global<Shared<i32>>>(None, None).is_err())
    }

    #[test]
    fn resolve_global_shared() {
        let memory_domain = Arc::new(MemoryDomain::new());
        
        assert!(memory_domain.insert(ResourceId::from_raw_heap::<i32>(), Resource::dummy(1)).unwrap().is_none());

        assert_eq!(***memory_domain.resolve::<Global<Shared<i32>>>(None, None).unwrap(), 1 as i32);
    }

    #[test]
    fn resolve_global_memory() {
        let program_id = ProgramId::from("Foo");
        let memory_domain = Arc::new(MemoryDomain::new());
        
        assert!(memory_domain.insert(ResourceId::from_raw_heap::<i32>(), Resource::dummy(1)).unwrap().is_none());

        let memory = Memory::new();
        assert!(memory.insert_program(program_id.clone(), memory_domain, None));

        memory.insert(None, None, None, 2 as i32);

        assert_eq!(***memory.resolve::<Global<Shared<i32>>>(Some(&program_id), None, None, None).unwrap().unwrap(), 2 as i32);
        assert_eq!(**memory.resolve::<Shared<i32>>(Some(&program_id), None, None, None).unwrap().unwrap(), 1 as i32);
        assert_eq!(**memory.resolve::<Shared<i32>>(None, None, None, None).unwrap().unwrap(), 2 as i32);
        assert_eq!(***memory.resolve::<Global<Shared<i32>>>(None, None, None, None).unwrap().unwrap(), 2 as i32);
    }
}