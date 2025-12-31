use std::cell::UnsafeCell;

pub mod inner_registry;

pub use inner_registry::{
    OperatedRegistry,
    RegistryOperator
};

use crate::registry::owned_registry::{reception::host::access_map::access::Access, registry_result::RegistryResult};

pub trait RegistryManager {
    type RegistryOperator: RegistryOperator;
}

pub struct ManagedRegistry<R: RegistryManager> {
    registry: UnsafeCell<OperatedRegistry<R::RegistryOperator>>
}

impl<R: RegistryManager> ManagedRegistry<R> {
    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_inner(&self) -> &OperatedRegistry<R::RegistryOperator> {
        unsafe { & *self.registry.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_mut(&self) -> &mut OperatedRegistry<R::RegistryOperator> {
        unsafe { &mut *self.registry.get() }
    }

    pub fn get<T: 'static, A: Access>(
        &self, 
        resource_id: &<<R as RegistryManager>::RegistryOperator as RegistryOperator>::ResourceId,
        access: &A,
    ) -> RegistryResult<'_, T> {
        unsafe { self.get_inner().get(resource_id, access) }
    }
}

impl<R: RegistryManager> Default for ManagedRegistry<R> {
    fn default() -> Self {
        Self {
            registry: UnsafeCell::new(OperatedRegistry::default())
        }
    }
}