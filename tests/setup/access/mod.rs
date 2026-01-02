use aion_reactor::prelude::Accessor;

use crate::setup::{StoredResource, access::{access_result::AccessResult, borrow_type::BorrowType}};

pub mod access_result;
pub mod borrow_type;

#[derive(Debug, PartialEq)]
pub enum Access {
    Shared(usize),
    Unique,
    Owned,
    Replace,
}

impl Access {
    pub fn borrow_type(&self) -> BorrowType {
        match self {
            Access::Shared(0) => BorrowType::Instant,

            Access::Owned => BorrowType::Instant,
            Access::Replace => BorrowType::Instant,

            Access::Shared(_) => BorrowType::Held,
            Access::Unique => BorrowType::Held,
        }
    }
}

impl Accessor for Access {
    type StoredResource = StoredResource;
    type Resource = i32;

    type AccessResult<'a, T> = AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool {
        match (self.borrow_type(), other.borrow_type()) {
            (BorrowType::Held, BorrowType::Held) => {
                match (self, other) {
                    (Access::Shared(_), Access::Shared(_)) => true,
                    _ => false,
                }
            },

            (BorrowType::Held, BorrowType::Instant) => !other.can_remove_resource(),

            (BorrowType::Instant, _) => true
        }
    }

    fn split_access(&mut self, other: &Self) {
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n -= m,     
            _ => ()
        }     
    }

    fn can_remove_resource(&self) -> bool {
        match self {
            Access::Replace => true,
            _ => false
        }
    }

    fn merge_access(&mut self, other: Self) {
        if self.borrow_type() == BorrowType::Instant {
            *self = other;
            return
        }

        assert_eq!(self.borrow_type(), BorrowType::Held);

        if other.borrow_type() == BorrowType::Instant {
            assert_ne!(other, Access::Replace, "Tried replacing a held borrow");

            return;
        }

        assert_eq!(other.borrow_type(), BorrowType::Held);

        match (self.borrow_type(), other.borrow_type()) {
            (BorrowType::Held, BorrowType::Held) => {
                match (self, other) {
                    (Access::Shared(n), Access::Shared(m)) => *n += m,
                    _ => panic!("Tried merging unique held accesses")
                }
            },
            _ => unreachable!()
        }
    }

    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource> {
        match self {
            Access::Shared(_) => AccessResult::Shared(&resource.0),
            Access::Unique => AccessResult::Unique(&resource.0),
            Access::Owned => AccessResult::Owned(resource.0.clone()),
            Access::Replace => panic!("Tried Accessing with `Replace`"),
        }
    }

    fn remove<'a>(&self, resource: Self::StoredResource) -> Self::AccessResult<'a, Self::StoredResource> {
        AccessResult::Owned(resource)
    }
}