use std::collections::HashMap;

use tracing::{Level, span};

use crate::prelude::{Accessor, OperatedRegistryAccessResult, ResourceKey};

pub mod registry_results;
pub mod resource_key;

pub struct OperatedRegistry<ResourceId, StoredResource> {
    registry: HashMap<ResourceId, StoredResource>
}

// temp solution
// box prevents dangling pointers from reallocation
// does mean always heap allocated :/
// stack allocated would require fixed size hashmap or some other reallocation aware struct
impl<
    ResourceId: ResourceKey, 
    StoredResource
> OperatedRegistry<ResourceId, Box<StoredResource>> {
    pub fn access<Access: Accessor<StoredResource = StoredResource>>(
        &self, 
        resource_id: &ResourceId,
        access: &Access
    ) -> OperatedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> {
        let span = span!(Level::DEBUG, "Operated Registry Access");
        let _enter = span.enter();

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
        let span = span!(Level::DEBUG, "Operated Registry Accessed Replacement");
        let _enter = span.enter();

        let old_resource = match resource {
            Some(new_resource) => {
                if !access.can_insert() || (self.registry.contains_key(&resource_id) && !access.can_remove()) {
                    return OperatedRegistryAccessResult::AccessFailure;
                }

                access.insert(&new_resource);
                // todo! if this insert would reallocate && there are concurrent accesses, FAIL
                // strategies:
                // use smart pointers around all resources (current implementation)
                // use fixed size hashmap
                // attempt to resize whenever it can, so sometimes will fail but hopefully rare
                self.registry.insert(resource_id, Box::new(new_resource))
            },
            None => {
                if self.registry.contains_key(&resource_id) && !access.can_remove() {
                    return OperatedRegistryAccessResult::AccessFailure;
                }

                self.registry.remove(&resource_id)
            }
        };

        match old_resource {
            Some(old_resource) => OperatedRegistryAccessResult::Found(access.remove(*old_resource)),
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