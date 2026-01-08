use aion_reactor::prelude::ManagedRegistry;

use crate::default_implementation::{init_tracing, prelude::{ResourceId, StoredResource}};

pub mod operated_registry;

pub fn setup_managed_registry() -> ManagedRegistry<ResourceId, Box<StoredResource>> {
    init_tracing();
    ManagedRegistry::default()
}