use std::{collections::HashMap, hash::Hash};

use crate::registry::owned_registry::{reception::host::Accessor, managed_registry::operated_registry::{registry_results::RegistryOperatorAccessResult}};

pub mod registry_results;

pub trait ResourceKey: Hash + PartialEq + Eq {}

pub struct OperatedRegistry<ResourceId, Resource> {
    registry: HashMap<ResourceId, Resource>
}

impl<
    ResourceId: ResourceKey, 
    Resource
> OperatedRegistry<ResourceId, Resource> {
    pub fn access<Access: Accessor<StoredResource = Resource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access
    ) -> RegistryOperatorAccessResult<Access::AccessResult<'_, Access::Resource>> {
        if let Some(resource) = self.registry.get(resource_id) {
            RegistryOperatorAccessResult::Found(access.access(resource))
        } else {
            RegistryOperatorAccessResult::ResourceNotFound
        }
    }

    pub fn replace(
        &mut self,
        resource_id: ResourceId,
        resource: Resource
    ) -> Option<Resource> {
        self.registry.insert(resource_id, resource)
    }
}

impl<ResourceId, Resource> Default for OperatedRegistry<ResourceId, Resource> {
    fn default() -> Self {
        Self {
            registry: HashMap::new()
        }
    }
}