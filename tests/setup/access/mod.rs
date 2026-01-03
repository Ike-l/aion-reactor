use aion_reactor::prelude::Accessor;
use tracing::{Level, event};

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

    pub fn can_remove(&self) -> bool {
        self.borrow_type() == BorrowType::Instant
    }
}

impl Accessor for Access {
    type StoredResource = StoredResource;
    type Resource = i32;

    type AccessResult<'a, T> = AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool {
        event!(Level::DEBUG, "Can Access");
        let r = match (self.borrow_type(), other.borrow_type()) {
            (BorrowType::Held, BorrowType::Held) => {
                match (self, other) {
                    (Access::Shared(_), Access::Shared(_)) => true,
                    _ => false,
                }
            },

            (BorrowType::Held, BorrowType::Instant) => *other != Access::Replace,

            (BorrowType::Instant, _) => true
        };
        // println!("Result: {r:?}");

        r
    }

    fn split_access(&mut self, other: &Self) {
        event!(Level::DEBUG, "Splitting Access");
        match (self, other) {
            (Access::Shared(n), Access::Shared(m)) => *n -= m,     
            _ => ()
        }     
    }

    fn merge_access(&mut self, other: Self) {
        event!(Level::DEBUG, "Merging Access");
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
        event!(Level::DEBUG, "Accessing Resource: {resource:?}");
        match self {
            Access::Shared(_) => AccessResult::Shared(&resource.0),
            Access::Unique => AccessResult::Unique(&resource.0),
            Access::Owned => AccessResult::Owned(resource.0.clone()),
            Access::Replace => panic!("Tried Accessing with `Replace`"),
        }
    }

    fn remove<'a>(&self, resource: Self::StoredResource) -> Self::AccessResult<'a, Self::StoredResource> {
        event!(Level::DEBUG, "Removing Resource: {resource:?}");
        AccessResult::Owned(resource)
    }

    fn insert<'a>(&self, resource: &'a Self::StoredResource) {
        event!(Level::DEBUG, "Removing Resource: {resource:?}");
    }
}