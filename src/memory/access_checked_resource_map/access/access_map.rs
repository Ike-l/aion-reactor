use std::collections::HashMap;

use crate::memory::access_checked_resource_map::{access::Access, resource::ResourceId, ResolveError};


#[derive(Debug, Default)]
pub struct AccessMap(HashMap<ResourceId, Access>);

impl AccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (ResourceId, Access)> {
        self.0.drain()
    }

    pub fn conflicts(&self, other: &Self) -> bool {
        other.0.iter().any(|(ty, acc)| {
            if let Some(access) = self.0.get(ty) {
                match (acc, access) {
                    (Access::Shared(_), Access::Shared(_)) => false,
                    _ => true
                }
            } else {
                false
            }
        })
    }

    pub fn access_shared<T: Into<ResourceId>>(&mut self, resource_id: T) -> Result<(), ResolveError> {
        let resource_id = resource_id.into();
        match self.0.entry(resource_id.clone()).or_insert(Access::Shared(0)) {
            Access::Shared(n) => Ok(*n += 1),
            Access::Unique => Err(ResolveError::ConflictingAccess(resource_id))
        }
    }

    pub fn access_unique<T: Into<ResourceId>>(&mut self, resource_id: T) -> Result<(), ResolveError> {
        let resource_id = resource_id.into();
        if self.0.contains_key(&resource_id) {
            return Err(ResolveError::ConflictingAccess(resource_id));
        }

        self.0.insert(resource_id, Access::Unique);
        Ok(())
    }
}