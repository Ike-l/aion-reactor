use std::collections::HashMap;

use crate::prelude::{Accessor, OperatedRegistryAccessResult, ResourceKey};

pub mod registry_results;
pub mod resource_key;

pub struct OperatedRegistry<ResourceId, StoredResource> {
    registry: HashMap<ResourceId, StoredResource>
}

impl<
    ResourceId: ResourceKey, 
    StoredResource
> OperatedRegistry<ResourceId, StoredResource> {
    pub fn access<Access: Accessor<StoredResource = StoredResource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access
    ) -> OperatedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> {
        if let Some(resource) = self.registry.get(resource_id) {
            OperatedRegistryAccessResult::Found(access.access(resource))
        } else {
            OperatedRegistryAccessResult::ResourceNotFound
        }
    }

    pub fn replace(
        &mut self,
        resource_id: ResourceId,
        resource: StoredResource
    ) -> Option<StoredResource> {
        self.registry.insert(resource_id, resource)
    }
}

impl<ResourceId, StoredResource> Default for OperatedRegistry<ResourceId, StoredResource> {
    fn default() -> Self {
        Self {
            registry: HashMap::new()
        }
    }
}