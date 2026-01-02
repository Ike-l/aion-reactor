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

    pub fn accessed_replace<Access: Accessor<StoredResource = StoredResource>>(
        &mut self,
        resource_id: ResourceId,
        access: &Access,
        resource: Option<StoredResource>
    ) -> OperatedRegistryAccessResult<Access::AccessResult<'_, Access::StoredResource>> {
        let old_resource = match resource {
            Some(new_resource) => {
                access.insert(&new_resource);
                self.registry.insert(resource_id, new_resource)
            },
            None => self.registry.remove(&resource_id)
        };

        match old_resource {
            Some(old_resource) => OperatedRegistryAccessResult::Found(access.remove(old_resource)),
            None => OperatedRegistryAccessResult::ResourceNotFound,
        }
    }
}

impl<ResourceId, StoredResource> Default for OperatedRegistry<ResourceId, StoredResource> {
    fn default() -> Self {
        Self {
            registry: HashMap::new()
        }
    }
}