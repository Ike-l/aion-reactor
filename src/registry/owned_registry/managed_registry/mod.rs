use std::cell::UnsafeCell;

pub mod operated_registry;
pub mod registry_results;

pub use operated_registry::{
    OperatedRegistry,
};

use crate::registry::owned_registry::{reception::host::Accessor, managed_registry::{registry_results::ManagedRegistryAccessResult, operated_registry::{ResourceKey, registry_results::RegistryOperatorAccessResult}}};

pub struct ManagedRegistry<ResourceId, Resource> {
    registry: UnsafeCell<OperatedRegistry<ResourceId, Resource>>
}

impl<
    ResourceId: ResourceKey, 
    Resource
> ManagedRegistry<ResourceId, Resource> {
    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_inner(&self) -> &OperatedRegistry<ResourceId, Resource> {
        unsafe { & *self.registry.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_mut(&self) -> &mut OperatedRegistry<ResourceId, Resource> {
        unsafe { &mut *self.registry.get() }
    }

    pub unsafe fn access<Access: Accessor<StoredResource = Resource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access,
    ) -> ManagedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> {
        unsafe { 
            match self.get_inner().access(resource_id, access) {
                RegistryOperatorAccessResult::Found(access_result) => ManagedRegistryAccessResult::Found(access_result),
                RegistryOperatorAccessResult::ResourceNotFound => ManagedRegistryAccessResult::ResourceNotFound,
            }
        }
    }

    pub unsafe fn replace(
        &self,
        resource_id: ResourceId,
        resource: Resource
    ) -> Option<Resource> {
        unsafe {
            self.get_inner_mut().replace(resource_id, resource)
        }
    }
}

impl<ResourceId, Resource> Default for ManagedRegistry<ResourceId, Resource> {
    fn default() -> Self {
        Self {
            registry: UnsafeCell::new(OperatedRegistry::default())
        }
    }
}