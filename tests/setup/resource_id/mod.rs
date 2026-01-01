use std::any::TypeId;

use aion_reactor::registry::owned_registry::{reception::host::access_map::AccessKey, managed_registry::operated_registry::ResourceKey};

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum ResourceId {
    Labelled(String),
    Raw(TypeId)
}

impl ResourceKey for ResourceId {}
impl AccessKey for ResourceId {}