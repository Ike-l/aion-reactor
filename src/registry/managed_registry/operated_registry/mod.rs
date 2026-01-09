use std::collections::HashMap;

use tracing::{Level, span};

use crate::prelude::{Accessor, OperatedRegistryAccessResult, OperatedRegistryReplacementResult, ResourceKey};

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
    ) -> OperatedRegistryReplacementResult<Access::AccessResult<'_, Access::StoredResource>> {
        let span = span!(Level::DEBUG, "Operated Registry Accessed Replacement");
        let _enter = span.enter();

        let old_resource = match (resource, self.registry.contains_key(&resource_id), access.can_insert(), access.can_remove()) {
            // Remove
            (None, true, _, true) => self.registry.remove(&resource_id),
            (None, true, _, false) => return OperatedRegistryReplacementResult::AccessFailure,

            // Nothing
            (None, false, _, _) => return OperatedRegistryReplacementResult::NoOp,

            // Replace
            (Some(new_resource), true, true, true) => {
                access.insert(&new_resource);
                self.registry.insert(resource_id, Box::new(new_resource))
            },

            (Some(_), true, false, _) => return OperatedRegistryReplacementResult::AccessFailure,
            (Some(_), true, _, false) => return OperatedRegistryReplacementResult::AccessFailure,

            // Insert
            (Some(new_resource), false, true, _) => {
                access.insert(&new_resource);
                self.registry.insert(resource_id, Box::new(new_resource))
            },
            (Some(_), false, false, _) => return OperatedRegistryReplacementResult::AccessFailure,
        };

        match old_resource {
            Some(old_resource) => OperatedRegistryReplacementResult::Found(access.remove(*old_resource)),
            None => OperatedRegistryReplacementResult::ResourceNotFound,
        }
    }

    pub fn contains(
        &self,
        resource_id: &ResourceId
    ) -> bool {
        self.registry.contains_key(resource_id)
    }
}

impl<ResourceId, StoredResource> Default for OperatedRegistry<ResourceId, Box<StoredResource>> {
    fn default() -> Self {
        Self {
            registry: HashMap::new()
        }
    }
}