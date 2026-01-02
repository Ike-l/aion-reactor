use std::collections::HashMap;

use crate::prelude::{AccessKey, AccessPermission, Accessor};

pub mod access_map_permission;
pub mod accessor;
pub mod access_key;

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
            (None, Some(current_access)) => AccessPermission::Insert(current_access.can_replace_resource()),
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