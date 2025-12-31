use std::{collections::HashMap, hash::Hash};

use crate::registry::owned_registry::reception::host::access_map::{access::Access, access_map_permission::AccessMapPermission};

pub mod access_map_permission;

pub mod access;

pub trait Accessor {
    type AccessId: Hash + Eq + PartialEq;
    type Access: Access;
}

pub struct AccessMap<A: Accessor> {
    accesses: HashMap<A::AccessId, A::Access>
}

impl<A: Accessor> AccessMap<A> {
    pub fn permits_access(
        &self,
        access_id: &A::AccessId,
        access: &A::Access
    ) -> AccessMapPermission {
        if let Some(current_access) = self.accesses.get(access_id) {
            AccessMapPermission::Coexistence(access.can_coexit(current_access))
        } else {
            AccessMapPermission::UnknownAccessId
        }
    }
}

impl<A: Accessor> Default for AccessMap<A> {
    fn default() -> Self {
        Self {
            accesses: HashMap::new()
        }
    }
}