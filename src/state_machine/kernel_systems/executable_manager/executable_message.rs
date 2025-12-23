use crate::{memory::ResourceId, ecs::entity::EntityId};

#[derive(Clone)]
pub enum ExecutableMessage {
    ResourceId(ResourceId),
    ECS(EntityId)
}