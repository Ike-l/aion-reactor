use std::any::TypeId;

use aion_reactor::prelude::{AccessKey, ResourceKey};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ResourceId {
    Labelled(String),
    Raw(TypeId)
}

impl ResourceKey for ResourceId {}
impl AccessKey for ResourceId {}