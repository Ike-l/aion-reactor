#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntityId(hecs::Entity);

impl EntityId {
    pub fn new_hecs(entity: hecs::Entity) -> Self {
        Self(entity)
    }

    pub fn get_hecs(&self) -> Option<&hecs::Entity> {
        Some(&self.0)
    }
}