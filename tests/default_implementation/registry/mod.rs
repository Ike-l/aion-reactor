
use aion_reactor::prelude::Registry;

use crate::default_implementation::{init_tracing, prelude::{Access, KeyId, ReserverId, ResourceId, StoredResource}};

pub mod managed_registry;
pub mod reception;

pub fn setup_registry() -> Registry<ResourceId, ReserverId, Access, ResourceId, KeyId, Box<StoredResource>> {
    init_tracing();
    Registry::default()
}