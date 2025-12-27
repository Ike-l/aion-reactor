use crate::prelude::{EntityId, ResourceId};

#[derive(Debug, Clone)]
pub enum ExecutableMessage {
    ResourceId(ResourceId),
    ECS(EntityId)
}