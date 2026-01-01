use std::{collections::HashMap, hash::Hash};

use crate::registry::owned_registry::{reception::gate::gate_permission::GateAccessPermission, managed_registry::operated_registry::ResourceKey};

pub mod gate_permission;

pub trait Key: Hash + PartialEq + Eq {}

pub struct Gate<ResourceId, KeyId> {
    keys: HashMap<ResourceId, KeyId>
}

impl<
    ResourceId: ResourceKey, 
    KeyId: Key
> Gate<ResourceId, KeyId> {
    pub fn allows_passage(&self, resource_id: &ResourceId, key: Option<&KeyId>) -> GateAccessPermission {
        match self.keys.get(resource_id) {
            None => GateAccessPermission::Unlocked,
            Some(locked_key) => {
                match key {
                    None => GateAccessPermission::Denied,
                    Some(different_key) if different_key != locked_key => GateAccessPermission::Denied,
                    Some(same_key) if same_key == locked_key => GateAccessPermission::Allowed,
                    _ => unreachable!()
                }
            },
        }
    }
}

impl<ResourceId, Key> Default for Gate<ResourceId, Key> {
    fn default() -> Self {
        Self {
            keys: HashMap::new()
        }
    }
}