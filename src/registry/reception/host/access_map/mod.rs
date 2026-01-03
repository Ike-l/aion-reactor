use std::collections::HashMap;

use tracing::{Level, event, field, span};

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
        access: &Access
    ) -> AccessPermission {
        let span = span!(Level::DEBUG, "AccessMap Permits Access", current_access =? field::Empty);
        let _enter = span.enter();

        match self.accesses.read().get(access_id) {
            Some(current_access) => {
                span.record("current_access", format!("{current_access:?}"));
                AccessPermission::Access(current_access.can_access(access))
            }
            None => AccessPermission::UnknownAccessId,
        }
    }

    pub fn remove_access(
        &self,
        access_id: &AccessId,
        access: &Access
    ) {
        let span = span!(Level::DEBUG, "AccessMap Remove Access", current_access =? field::Empty);
        let _enter = span.enter();

        if let Some(current_access) = self.accesses.write().get_mut(access_id) {
            span.record("current_access", format!("{current_access:?}"));
            current_access.split_access(access);
        } else {
            event!(Level::WARN, "UnknownAccessId")
        }
    }

    pub fn record_access(
        &self, 
        access_id: AccessId,
        new_access: Access
    ) {
        let span = span!(Level::DEBUG, "AccessMap Record Access", current_access =? field::Empty);
        let _enter = span.enter();

        if let Some(current_access) = self.accesses.write().get_mut(&access_id) {
            span.record("current_access", format!("{current_access:?}"));
            current_access.merge_access(new_access);
        } else {
            event!(Level::INFO, "Inserting Access");
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