use aion_reactor::prelude::Accessor;

use crate::setup::{StoredResource, access::access_result::AccessResult};

pub mod access_result;

pub enum Access {
    Shared(usize),
    Unique,
    Owned
}

impl Accessor for Access {
    type StoredResource = StoredResource;
    type Resource = i32;

    type AccessResult<'a, T> = AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool {
        match (self, other) {
            (Access::Owned, Access::Owned) |
            (Access::Owned, Access::Unique) |
            (Access::Owned, Access::Shared(_)) |
            (Access::Unique, Access::Owned) |
            (Access::Shared(_), Access::Owned) |
            
            (Access::Shared(_), Access::Shared(_)) => true,     
            
            (Access::Unique, Access::Unique) |
            (Access::Unique, Access::Shared(_)) |
            (Access::Shared(_), Access::Unique) => false,
        }      
    }

    fn split_access(&mut self, other: &Self) {
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n -= m,     
            
            (Access::Unique, Access::Owned) |
            (Access::Owned, Access::Owned) |
            (Access::Shared(_), Access::Owned) |                        

            (Access::Owned, Access::Unique) |
            (Access::Owned, Access::Shared(_)) => (),

            (Access::Unique, Access::Unique) |
            (Access::Unique, Access::Shared(_)) |
            (Access::Shared(_), Access::Unique) => unreachable!()
        }     
    }

    fn can_replace_resource(&self) -> bool {
        match self {
            Access::Shared(_) => false,
            Access::Unique => false,
            Access::Owned => true,
        }
    }

    fn merge_access(&mut self, other: Self) {
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n += m,
            
            (Access::Shared(_), Access::Owned) |
            (Access::Unique, Access::Owned) |
            (Access::Owned, Access::Owned) => (),

            (Access::Shared(_), Access::Unique) |
            (Access::Unique, Access::Shared(_)) |
            (Access::Unique, Access::Unique) => unreachable!(),

            (owned @ Access::Owned, rhs @ _) => *owned = rhs,
        }
    }

    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource> {
        match self {
            Access::Shared(_) => AccessResult::Shared(&resource.0),
            Access::Unique => AccessResult::Unique(&resource.0),
            Access::Owned => AccessResult::Owned(resource.0.clone()),
        }
    }
}