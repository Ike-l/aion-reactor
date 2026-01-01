use aion_reactor::registry::owned_registry::reception::host::Accessor;

use crate::setup::{Resource, access::access_result::AccessResult};

pub mod access_result;

pub enum Access {
    Shared(usize),
    Unique,
}

impl Accessor for Access {
    type StoredResource = Resource;
    type Resource = i32;

    type AccessResult<'a, T> = AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool {
        match (self, other) {
            (Access::Shared(_), Access::Shared(_)) => true,
            _ => false
        }      
    }

    fn split_access(&mut self, other: &Self) {
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n -= m,
            _ => panic!("Cannot merge")
        }
    }

    fn can_insert(&self) -> bool {
        false
    }

    fn merge_access(&mut self, other: Self) {
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n += m,
            _ => panic!("Cannot merge")
        }
    }

    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource> {
        match self {
            Access::Shared(_) => AccessResult::Shared(&resource.0),
            _ => AccessResult::Fail,
        }
    }
}