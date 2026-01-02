use std::cell::UnsafeCell;

use crate::prelude::{Accessor, ManagedRegistryAccessResult, OperatedRegistry, OperatedRegistryAccessResult, ResourceKey};

pub mod operated_registry;
pub mod registry_results;

pub struct ManagedRegistry<ResourceId, StoredResource> {
    registry: UnsafeCell<OperatedRegistry<ResourceId, StoredResource>>
}

impl<
    ResourceId: ResourceKey, 
    StoredResource
> ManagedRegistry<ResourceId, StoredResource> {
    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_inner(&self) -> &OperatedRegistry<ResourceId, StoredResource> {
        unsafe { & *self.registry.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_mut(&self) -> &mut OperatedRegistry<ResourceId, StoredResource> {
        unsafe { &mut *self.registry.get() }
    }

    pub unsafe fn access<Access: Accessor<StoredResource = StoredResource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access,
    ) -> ManagedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> {
        unsafe { 
            match self.get_inner().access(resource_id, access) {
                OperatedRegistryAccessResult::Found(access_result) => ManagedRegistryAccessResult::Found(access_result),
                OperatedRegistryAccessResult::ResourceNotFound => ManagedRegistryAccessResult::ResourceNotFound,
            }
        }
    }

    pub unsafe fn replace(
        &self,
        resource_id: ResourceId,
        resource: StoredResource
    ) -> Option<StoredResource> {
        unsafe {
            self.get_inner_mut().replace(resource_id, resource)
        }
    }
}

impl<ResourceId, StoredResource> Default for ManagedRegistry<ResourceId, StoredResource> {
    fn default() -> Self {
        Self {
            registry: UnsafeCell::new(OperatedRegistry::default())
        }
    }
}