use std::{collections::HashMap, hash::Hash};

use crate::registry::owned_registry::reception::host::access_map::access_map_permission::{AccessPermission};

pub mod access_map_permission;

pub trait Accessor {
    type StoredResource;
    type Resource: 'static;
    type AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool;
    fn can_insert(&self) -> bool;

    fn merge_access(&mut self, other: Self);
    fn split_access(&mut self, other: &Self);

    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource>;
}

pub trait AccessKey: Hash + PartialEq + Eq + Clone {}

pub struct AccessMap<AccessId, Access> {
    accesses: parking_lot::RwLock<HashMap<AccessId, Access>>
}

impl<AccessId: AccessKey, Access: Accessor> AccessMap<AccessId, Access> {
    pub fn permits_access(
        &self,
        access_id: &AccessId,
        access: Option<&Access>
    ) -> AccessPermission {
        match (access, self.accesses.read().get(access_id)) {
            (Some(new_access), Some(current_access)) => AccessPermission::Access(current_access.can_access(new_access)),
            (None, Some(current_access)) => AccessPermission::Insert(current_access.can_insert()),
            (_, None) => AccessPermission::UnknownAccessId,
        }
    }

    pub fn remove_access(
        &self,
        access_id: &AccessId,
        access: &Access
    ) {
        if let Some(current_access) = self.accesses.write().get_mut(access_id) {
            current_access.split_access(access)
        }
    }

    pub fn record_access(
        &self, 
        access_id: AccessId,
        new_access: Access
    ) {
        if let Some(current_access) = self.accesses.write().get_mut(&access_id) {
            current_access.merge_access(new_access);
        } else {
            self.accesses.write().insert(access_id, new_access);
        }
    }
}

impl<AccessId, Access> Default for AccessMap<AccessId, Access> {
    fn default() -> Self {
        Self {
            accesses: parking_lot::RwLock::new(HashMap::new())
        }
    }
}