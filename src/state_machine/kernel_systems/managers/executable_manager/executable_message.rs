use crate::prelude::{EntityId, ResourceId};

#[derive(Clone)]
pub enum ExecutableMessage {
    ResourceId(ResourceId),
    ECS(EntityId)
}