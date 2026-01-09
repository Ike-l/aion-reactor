use std::cell::UnsafeCell;

use tracing::{Level, span};

use crate::prelude::{Accessor, ManagedRegistryAccessResult, ManagedRegistryReplacementResult, OperatedRegistry, OperatedRegistryAccessResult, OperatedRegistryReplacementResult, ResourceKey};

pub mod operated_registry;
pub mod registry_results;

pub struct ManagedRegistry<ResourceId, StoredResource> {
    registry: UnsafeCell<OperatedRegistry<ResourceId, StoredResource>>
}

impl<
    ResourceId: ResourceKey, 
    StoredResource
> ManagedRegistry<ResourceId, Box<StoredResource>> {
    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_inner(&self) -> &OperatedRegistry<ResourceId, Box<StoredResource>> {
        unsafe { & *self.registry.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_mut(&self) -> &mut OperatedRegistry<ResourceId, Box<StoredResource>> {
        unsafe { &mut *self.registry.get() }
    }

    pub unsafe fn access<Access: Accessor<StoredResource = StoredResource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access,
    ) -> ManagedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> {
        let span = span!(Level::DEBUG, "Managed Registry Access");
        let _enter = span.enter();

        unsafe { 
            match self.get_inner().access(resource_id, access) {
                OperatedRegistryAccessResult::Found(access_result) => ManagedRegistryAccessResult::Found(access_result),
                OperatedRegistryAccessResult::ResourceNotFound => ManagedRegistryAccessResult::ResourceNotFound,
            }
        }
    }

    pub unsafe fn accessed_replacement<Access: Accessor<StoredResource = StoredResource>>(
        &self,
        resource_id: ResourceId,
        resource: Option<StoredResource>,
        access: &Access,
    ) -> ManagedRegistryReplacementResult<<Access as Accessor>::AccessResult<'_, StoredResource>> {
        let span = span!(Level::DEBUG, "Managed Registry Accessed Replacement");
        let _enter = span.enter();

        unsafe {
            match self.get_inner_mut().accessed_replace(resource_id, access, resource) {
                OperatedRegistryReplacementResult::Found(access_result) => ManagedRegistryReplacementResult::Found(access_result),
                OperatedRegistryReplacementResult::ResourceNotFound => ManagedRegistryReplacementResult::ResourceNotFound,
                OperatedRegistryReplacementResult::AccessFailure => ManagedRegistryReplacementResult::AccessFailure,
                OperatedRegistryReplacementResult::NoOp => ManagedRegistryReplacementResult::NoOp,
            }
        }
    }

    pub fn contains(
        &self,
        resource_id: &ResourceId
    ) -> bool {
        unsafe { self.get_inner().contains(resource_id) }
    }
}

impl<ResourceId, StoredResource> Default for ManagedRegistry<ResourceId, Box<StoredResource>> {
    fn default() -> Self {
        Self {
            registry: UnsafeCell::new(OperatedRegistry::default())
        }
    }
}